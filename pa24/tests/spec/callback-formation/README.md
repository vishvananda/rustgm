# Ordinary callable initializer inference

Two independently accepted Rust 1.98 object inputs exercise a generic FnOnce-bound constructor with a default function-pointer parameter, in a free function and an ordinary trait impl. Both full LowIR outcomes pass. Their unused generic bodies are declared compilation contracts; actual initializer execution is covered by the PA25 const counterpart.

Two new references were independently reviewed and generated. The PA19 ordinary function-pointer minimum and all existing references remain separate and unchanged.
