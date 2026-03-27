// Copyright 2025 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Recover Poison example - recovering without `Clone`
//!
//! This example shows how to retrieve the internal error type

use std::error::Error;

use derive_more::Display;
use exn::Result;
use exn::ResultExt;

fn main() -> Result<(), MainError> {
    app::run(500).or_raise(|| MainError)?;
    Ok(())
}

#[derive(Debug, Display)]
#[display("fatal error occurred in application")]
struct MainError;
impl std::error::Error for MainError {}

mod app {
    use exn::bail;

    use super::*;

    pub fn run(id: u64) -> Result<u64, AppError> {
        orchestrator::run_separate(id).or_raise(|| AppError)
    }

    #[derive(Debug, Display)]
    #[display("could not resolve answer")]
    pub struct AppError;
    impl std::error::Error for AppError {}
}

mod orchestrator {
    use std::sync::{Arc, Mutex};

    use exn::bail;

    use super::*;

    pub fn run_separate(val: u64) -> Result<u64, OrchestrateError> {
        let v = Arc::new(Mutex::new(val));
        let v1 = v.clone();
        std::thread::spawn(|| op::add(v1, 42))
            .join()
            .unwrap()
            .or_raise(|| OrchestrateError)?;
        println!("{v:?}");
        // spawn a thread that panics
        match std::thread::spawn(|| op::fail(v)).join() {
            Err(e) => {
                eprintln!("operation failed");
                bail!(OrchestrateError)
            }
            Ok(_) => return Ok(42),
        }
    }

    #[derive(Debug, Display)]
    #[display("Error while running tasks in parallel")]
    pub struct OrchestrateError;
    impl std::error::Error for OrchestrateError {}
}

mod op {
    use std::sync::{Arc, Mutex};

    use exn::bail;

    use super::*;

    pub fn fail(value: Arc<Mutex<u64>>) -> Result<(), OpError> {
        let l = value.lock();
        if let Ok(v) = l {
            panic!("Panicked while locking")
        } else {
            bail!(OpError::Poisoned)
        }
    }

    pub fn add(a: Arc<Mutex<u64>>, b: u64) -> Result<(), OpError> {
        let l = a.lock();
        if let Ok(mut v) = l {
            *v += b;
            Ok(())
        } else {
            Err(OpError::Poisoned.into())
        }
    }

    #[derive(Debug, Display)]
    pub enum OpError {
        Poisoned,
        Failed,
    }
    impl Error for OpError {}
}
