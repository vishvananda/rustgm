#![feature(no_core)]#![no_core]
extern crate base; struct Local; impl base::outer::Open for Local {} impl base::outer::Visible for Local {}
