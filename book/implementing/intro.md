# Implementing Try

I like the trait for two reasons:

1. I don't like it when std can do stuff I can't, `Try` opens up the power and versatility of `?` to my own code.
1. I don't like writing extra code when it feels like I'm just working _around_ language constraints. `Try` lets me add `impl`s directly to a custom `Result` type or flatten nested constructs inside `Option`s & `Result`s
