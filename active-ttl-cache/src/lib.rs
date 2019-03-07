use std::hash::Hash;
use std::collections::HashMap;
use crossbeam_channel::{Sender, bounded, unbounded};
use std::marker::PhantomData;
use std::time::Duration;
use std::thread::{sleep, spawn};
use std::ops::Deref;

pub struct Entry<K: Clone, V> {
    notice: Sender<Event<K, V>>,
    inner: V,
    key: K,
}

impl<K, V> Deref for Entry<K, V>
    where K: Clone,
          V: Sized
{
    type Target = V;

    #[inline]
    fn deref(&self) -> &V {
        &self.inner
    }
}

impl<K, V> Drop for Entry<K, V>
    where K: Clone {
    fn drop(&mut self) {
        self.notice.send(Event::Decr(self.key.clone()))
            .expect("Internal cache error: drop");
    }
}

enum Event<K, V> {
    Get(K, Sender<Option<V>>),
    Decr(K),
    Expire(K),
}

#[derive(Clone)]
pub struct Handle<K, V> {
    sender: Sender<Event<K, V>>,
    pk: PhantomData<K>,
    pv: PhantomData<V>,
}

impl<K, V> Handle<K, V>
    where K: Clone {
    fn new(sender: Sender<Event<K, V>>) -> Self {
        Self { sender, pk: PhantomData, pv: PhantomData }
    }

    pub fn get(&self, key: K) -> Option<Entry<K, V>> {
        let (s, r) = bounded(1);
        self.sender.send(Event::Get(key.clone(), s)).expect("Internal cache error: get send");
        let notice = self.sender.clone();
        r.recv().expect("Internal cache error: get recv").map(|inner| {
            Entry { notice, inner, key }
        })
    }
}

struct Cache<K: Hash + Eq, V, F> {
    inner: HashMap<K, (usize, V)>,
    generator: F,
    pub ttd: Duration, // time-to-die
}

pub fn start<K, V, F>(generator: F, ttd: Duration) -> Handle<K, V>
    where K: 'static + Send + Clone + Hash + Eq,
          V: 'static + Send + Clone,
          F: 'static + Send + FnMut(K) -> Option<V>
{
    Cache::start(generator, ttd)
}

impl<K, V, F> Cache<K, V, F>
    where K: 'static + Send + Clone + Hash + Eq,
          V: 'static + Send + Clone,
          F: 'static + Send + FnMut(K) -> Option<V>
{
    fn start(generator: F, ttd: Duration) -> Handle<K, V> {
        let (s, r) = unbounded();
        let mains = s.clone();
        let handle = Handle::new(s);

        spawn(move || {
            let r = r;
            let mut cache = Self {
                inner: HashMap::new(),
                generator,
                ttd,
            };

            loop {
                match r.recv().expect("Internal cache error: main recv") {
                    Event::Get(k, s) => {
                        let v = cache.get(k);
                        s.send(v).expect("Internal cache error: main send");
                    },
                    Event::Decr(k) => {
                        let expk = k.clone();
                        if cache.decr(k) {
                            let s = mains.clone();
                            let ttd = cache.ttd.clone();
                            spawn(move || {
                                sleep(ttd);
                                s.send(Event::Expire(expk)).expect("Internal cache error: expire");
                            });
                        }
                    },
                    Event::Expire(k) => {
                        cache.expire(k);
                    }
                };
            }
        });

        handle
    }

    fn get(&mut self, k: K) -> Option<V> {
        if let Some((_, v)) = self.inner.get(&k) {
            let v = v.clone();
            self.incr(k);
            Some(v)
        } else if let Some(v) = (self.generator)(k.clone()) {
            self.inner.insert(k, (0, v.clone()));
            Some(v)
        } else {
            None
        }
    }

    fn incr(&mut self, k: K) {
        self.inner.entry(k).and_modify(|(rc, _)| { *rc += 1 });
    }

    fn decr(&mut self, k: K) -> bool {
        self.inner.entry(k.clone()).and_modify(|(rc, _)| { *rc -= 1 });
        if let Some((rc, _)) = self.inner.get(&k) {
            *rc == 0
        } else {
            false
        }
    }

    fn expire(&mut self, k: K) {
        if self.inner.get(&k).map(|(rc, _)| *rc).unwrap_or(1) == 0 {
            self.inner.remove(&k);
        }
    }
}
