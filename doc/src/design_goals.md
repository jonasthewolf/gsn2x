
# Design goals

I noticed that it might make sense to add some information about the goals I have set for myself for this project:

- Everything-as-code

  The tool should work in a continuos integration environment.
  The input format should be diff-able to support common git workflows with pull/merge requests.

- Simplicity

  I would like to keep things simple. Simple for me and others.

  That means the input format should be simple to learn and edit manually in any editor. 
  I also did not want invent a new DSL (domain-specific language) for that purpose.
  YAML (input file format) might not be the best format, but it serves as a good tradeoff for my purposes.
  Moreover, it can be parsed by other programs easily, too.

  To a certain extend this includes that the generated SVG files are not the smallest possible files.
  Readability and understandability to the user is higher rated, since the amount of memory required for the SVGs
  is still negligible to, e.g. a compiler installation.

- Standard conformance

  I would like the program output to be very close to the GSN standard.

  I don't want to redefine the semantics or add additional ones. 
  The standard was created so that as many people as possible have some common grounds.
  If I added new fancy stuff, everyone might have a different interpretation of that again.

  Standards conformance is also a goal for the generated SVG files and Markdown files.

- As few dependencies as possible

  Since I understand that this tool might be used in some corporate environment where usage of 
  free and open-source software might be limited, I try to keep the dependencies of this program 
  as few as possible.

  I used a very relaxed license and try to take care that it is compatible with those of the dependencies.
  
- Full control by the user how the diagram is laid out

  Creating an optimal rendering for all different use-cases is incredibly difficult.
  I failed in creating an algorithm that would allow for that. 
  When trying I recognized that it is anyhow preferable to give the user full control over how the diagram should look like.