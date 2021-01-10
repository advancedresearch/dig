#![deny(missing_docs)]

//! # Dig
//! A simple logistic environment primitive
//!
//! ### Motivation
//!
//! Study semantics of a simple environment primitive
//! that can be simulated using
//! many different physical systems.
//!
//! The benefit of an environment primitive is to easier communicate
//! ideas about control of environments among AI safety researchers.
//!
//! In some cases, control of complex environments might be reduced
//! to a mathematically equivalent problem using the environment primitive.
//!
//! ### Design
//!
//! There are two kinds of objects:
//!
//! - Containers
//! - Grabbers
//!
//! A container stores a volume of some unknown material.
//! A grabber moves some volume from one container to another.
//!
//! The grabber takes time to move,
//! in which time interval the moved resource is unavailable for other grabbers.
//! If the source container does not fill up the capacity of the grabber,
//! then all the remaining material in the container is moved by the grabber.
//! At the end of the time interval,
//! the resource is put in the target container.
//!
//! ### Internal vs External Environment
//!
//! This library only models the Internal Environment.
//!
//! All semantics about time, agents and terminal states are encoded externally.
//! This is called the External Environment.
//!
//! The distinction between internal and external is used to formalize the
//! language used to talk about safety in environments.

/// Stores volume of some material.
pub struct Container(pub f64);

impl Container {
    /// Adds some volume to the container.
    pub fn put(&mut self, v: f64) {
        self.0 += v;
    }

    /// Takes some volume from the container.
    pub fn take(&mut self, v: f64) -> f64 {
        if self.0 <= v {
            let v = self.0;
            self.0 = 0.0;
            v
        } else {
            self.0 -= v;
            v
        }
    }
}

/// Stores information about a grabber.
pub struct Grabber {
    /// The maximum volume capacity of the grabber.
    pub volume: f64,
    /// The time it takes for the grabber to transport material.
    pub time: f64,
    /// Stores source container ID.
    pub source: ContainerId,
    /// Stores target container ID.
    pub target: ContainerId,
}

/// Stores the grabber state.
pub struct GrabberState {
    /// The time remaining until the grabber is done.
    pub time: f64,
    /// The volume moved by the grabber.
    pub volume: f64,
}

/// Stores the Internal Environment.
pub struct Environment {
    /// Stores containers.
    pub containers: Vec<Container>,
    /// Stores grabbers.
    pub grabbers: Vec<Grabber>,
    /// Stores grabber states.
    pub grabber_states: Vec<GrabberState>,
}

/// Stores a container ID.
#[derive(Clone, Copy)]
pub struct ContainerId(pub usize);
/// Stores a grabber ID.
#[derive(Clone, Copy)]
pub struct GrabberId(pub usize);

impl Environment {
    /// Creates a new empty environment.
    pub fn new() -> Environment {
        Environment {
            containers: vec![],
            grabbers: vec![],
            grabber_states: vec![],
        }
    }

    /// Adds a new container to the environment.
    pub fn add_container(&mut self, c: Container) -> ContainerId {
        let id = self.containers.len();
        self.containers.push(c);
        ContainerId(id)
    }

    /// Adds a new grabber to the environment.
    pub fn add_grabber(&mut self, g: Grabber) -> GrabberId {
        let id = self.grabbers.len();
        self.grabbers.push(g);
        self.grabber_states.push(GrabberState {time: 0.0, volume: 0.0});
        GrabberId(id)
    }

    /// Activates a grabber, if not busy.
    ///
    /// Returns `Ok(())` if the grabber was activated.
    /// Returns `Err(())` if the grabber is busy.
    pub fn grab(&mut self, gid: GrabberId) -> Result<(), ()> {
        if self.grabber_states[gid.0].time == 0.0 {
            let g = &self.grabbers[gid.0];
            let v = g.volume;
            let v2 = self.containers[g.source.0].take(v);
            let s = &mut self.grabber_states[gid.0];
            s.volume = v2;
            s.time = g.time;
            Ok(())
        } else {
            Err(())
        }
    }

    /// Updates the environment with a time delta.
    pub fn update(&mut self, dt: f64) {
        let n = self.grabbers.len();
        for i in 0..n {
            let s = &mut self.grabber_states[i];
            s.time -= dt;
            if s.time <= 0.0 {
                let g = &self.grabbers[i];
                self.containers[g.target.0].put(s.volume);
                s.volume = 0.0;
                s.time = 0.0;
            }
        }
    }

    /// The volume of a container.
    pub fn volume_of_container(&self, c: ContainerId) -> f64 {
        self.containers[c.0].0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_from_container() {
        let mut a = Container(10.0);
        a.take(2.0);
        assert_eq!(a.0, 8.0);
    }

    #[test]
    fn test_environment() {
        let mut env = Environment::new();
        let a = env.add_container(Container(10.0));
        let b = env.add_container(Container(0.0));
        let ab = env.add_grabber(Grabber {
            source: a,
            target: b,
            time: 1.0,
            volume: 2.0,
        });
        assert_eq!(env.volume_of_container(a), 10.0);
        assert!(env.grab(ab).is_ok());
        assert_eq!(env.volume_of_container(a), 8.0);
        assert!(env.grab(ab).is_err());
        assert_eq!(env.volume_of_container(a), 8.0);
        env.update(0.5);
        assert!(env.grab(ab).is_err());
        env.update(0.5);
        assert!(env.grab(ab).is_ok());
        assert_eq!(env.volume_of_container(a), 6.0);
    }

    #[test]
    fn test_environment_remainder() {
        let mut env = Environment::new();
        let a = env.add_container(Container(1.0));
        let b = env.add_container(Container(0.0));
        let ab = env.add_grabber(Grabber {
            source: a,
            target: b,
            time: 1.0,
            volume: 2.0,
        });
        assert_eq!(env.volume_of_container(a), 1.0);
        assert!(env.grab(ab).is_ok());
        assert_eq!(env.volume_of_container(a), 0.0);
        assert_eq!(env.volume_of_container(b), 0.0);
        env.update(0.5);
        assert_eq!(env.volume_of_container(b), 0.0);
        env.update(0.5);
        assert_eq!(env.volume_of_container(b), 1.0);
    }

    #[test]
    fn test_environment_chain() {
        let mut env = Environment::new();
        let a = env.add_container(Container(1.0));
        let b = env.add_container(Container(0.0));
        let c = env.add_container(Container(0.0));
        let ab = env.add_grabber(Grabber {
            source: a,
            target: b,
            time: 1.0,
            volume: 1.0
        });
        let bc = env.add_grabber(Grabber {
            source: b,
            target: c,
            time: 1.0,
            volume: 1.0,
        });
        assert!(env.grab(ab).is_ok());

        env.update(1.0);
        assert_eq!(env.volume_of_container(a), 0.0);
        assert_eq!(env.volume_of_container(b), 1.0);
        assert_eq!(env.volume_of_container(c), 0.0);

        assert!(env.grab(bc).is_ok());

        env.update(1.0);
        assert_eq!(env.volume_of_container(a), 0.0);
        assert_eq!(env.volume_of_container(b), 0.0);
        assert_eq!(env.volume_of_container(c), 1.0);
    }
}

