
# History

I also noticed that (also for myself) it is good to note down some history of the project:

- It all started out in 2017 with the need for graphically representing some argumentation at work.
  I wrote a tiny Python script that used a jinja template to transform the YAML syntax
  into something Graphviz could understand.

  From there Graphviz could generate different output formats. That's where the `x` in `gsn2x` is from.

- It got obvious that some validation, especially on the uniqueness and reference resolution is needed
  to handle larger argumentation.
  
  I did not want to write those validations in Python, but in my favorite programming language Rust.
  I released the first Rust version in July 2021.
  
- I desperately tried adding the modular extension by convincing Graphviz to draw what I want, but I failed.
  I finally made decided to no longer output DOT, but directly generate SVGs from the program.
  This required writing a specialized version for rendering the tree on my own which ended up in version 2 
  finally released in April 2022.

Any feedback, especially the use-case in your company is very much appreciated.
