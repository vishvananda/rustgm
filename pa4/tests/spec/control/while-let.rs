{ let mut x = Some(3); while let Some(y) = x { x = None; y; } }
