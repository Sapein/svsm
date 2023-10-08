# Sapeint's Void System Manager (SVSM)

This is an attempt at writing something so that we can get somewhat reproducable void linux systems using something like Nix, but with it being native to and leveraging Void's infrastructure.



## QnA
### Why not just use Nix?  
While you can use Nix with Void Linux -- and it does seem to work alright -- there are three main issues
with using regular Nix on Void Linux:

1. It tends to break
2. There are many times issues with Nix integration with Void Linux
3. It does not leverage or use any of Void's features or packaging.

While these points aren't necessarily bad on their own (for example, Void may lack packages Nix has) I wanted
something that would integrate with Void Linux itself and allow me to leverage Void Linux's XBPS.

### Why is the language so weird?  
While the language is a bit weird, admittedly, it is designed to be a declarative
Domain Specific Language (DSL), and *not* an imperative general purpose programming language (GPPL). I, could
have used a regular programming language (Rust, Python, etc) as the configuration language. However,
I generally believe that encourages people to use bad patterns, and rely on transient state. I prefer fully
declarative solutions.


### What is the design of the language?
The language used in SVSM -- SVSL (Sapeint's Void System Language) -- is designed to meet the following
requirements:

1. Must be as fully declarative as is possible without making configuration tedious or difficult,
2. Must be easy to implement,
3. Must be easy to read and alter,
4. Must discourage bad practices and encourage good practices,

The first two are the most important. Being as declarative as possible allows for SVSM to determine *how*
to handle the configuration of the system. The end user should merely have to declare the state they desire
and then SVSM determines how best to get there. This is to allow for things to be relatively reproducible,
within the confines of Void Linux. 

Additionally, the more there is, the more there is to implement. Because I am doing this by myself, I needed
to have it be easy to implement so I can reasonably implement the language. See the next question for a bit
more on that.

### Why does the language have import, but not addition/function declaration/variables?  
Because of the requirement that it must be easy to implement, *and* also be relatively declarative I couldn't
reasonably justify Function Declarations *or* variables outside what exists currently, because doing so
would necessarily complicate the implementation. 

Furthermore, I believe that allowing non-implementation defined functions would break the declarative
nature of the system, as it is SVSM itself that is supposed to determine how to arrive at the state. *IF*
User-Defined Functions existed, then the system would have to defer to the user function to determine
how to arrive at that state, and it makes it practically impossible to determine how to reverse in
certain circumstances -- at least not without inspecting the function declaration, which itself
might require inspecting other functions defined, which would complicate things a lot. 

That being said, if a compelling case arises, or it can be shown to benefit using SVSM for users,
I am *not* against adding those into the language. However, I just could not reasonably come to the
conclusion that such features would be worth the time and complications necessary.