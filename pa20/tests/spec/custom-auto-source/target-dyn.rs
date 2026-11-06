#![feature(auto_traits)]
auto trait Auto{}trait Base{}impl Auto for dyn Base{}
