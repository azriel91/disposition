use dioxus::{
    html::GlobalAttributesExtension,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Link, Props},
};

/// Page that explains how `disposition` came to be.
#[component]
pub fn About() -> Element {
    rsx! {
        div {
            class: "\
                flex \
                flex-col \
                gap-6 \
                max-w-3xl \
                mx-auto \
                text-lg \
                [&_*]:mb-3
            ",
            Section {
                title: "Summary",
                ol {
                    class: "\
                        list-decimal \
                        pl-4 \
                    ",
                    li {
                        "Wanted to create an "
                        Link {
                            class: "text-blue-400 hover:text-blue-300",
                            to: "https://peace.mk",
                            new_tab: true,
                            "automation framework"
                        }
                        " that is truly understandable."
                    }
                    li {
                        "What's missing is suitable visibility of state and execution progress."
                    }
                    li {
                        "Diagrams are often inaccurate, cluttered, or ugly."
                    }
                    li {
                        "Stretched GraphViz to its limits in "
                        Link {
                            class: "text-blue-400 hover:text-blue-300",
                            to: "https://azriel.im/dot_ix",
                            new_tab: true,
                            code {
                                "dot_ix"
                            }
                        }
                        "."
                    }
                    li {
                        "Rewrote it in Rust."
                    }
                    li {
                        "AI is no substitute for empathy / engineering."
                    }
                }
            }
            Section {
                title: "Background",
                p {
                    "I began working on an automation framework called "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://peace.mk",
                        new_tab: true,
                        "🕊️ Peace"
                    }
                    " in 2022, as my version of \""
                    span { class: "italic", "I" }
                    " can build automation better\"."
                }
                p {
                    "For all good intentions about automation, there is always a trade-off with manual execution."
                }
                ul {
                    class: "list-disc pl-5",
                    li {
                        "Automation promises consistency, but requires every parameter to be understood upfront. Manual execution allows learning to happen during the process."
                    }
                    li {
                        "Automation promises performance, but a misunderstood process that stalls leaves one deeper in the dark than the light it offers."
                    }
                    li {
                        "Automation promises repeatability, but that repeatability is often built upon a process that is all-or-nothing in its execution. Manual execution, by nature, gives the user control over which step they proceed with next."
                    }
                }
                p {
                    "But I had already jumped on the Rust wagon, and in this world where decisions are often as trade-offs, the Rust response is to claim \""
                    span { class: "italic", "why not all" }
                    "\", and that's where this started."
                }
            }
            Section {
                title: "Visibility",
                p {
                    "More attention needs to be given to clear communication. This is especially true for an information dense source such as automation."
                }
                ul {
                    class: "list-disc pl-5",
                    li {
                        "Before it happens, the current state."
                    }
                    li {
                        "Before it happens, the target state."
                    }
                    li {
                        "Before it happens, the difference."
                    }
                    li {
                        "As it happens, what happened."
                    }
                    li {
                        "As it happens, what's happening."
                    }
                    li {
                        "As it happens, what's left to happen."
                    }
                    li {
                        "After it happens, what happened."
                    }
                }
                p {
                    "And present that information in a way I don't need to read (too much). Text works up to a point (usually the edge of my sanity), and beyond that a diagram works wonders."
                }
                p {
                    "It likely would've worked wonders before that as well."
                }
            }
            Section {
                title: "Existing Solutions",
                p {
                    "Inaccurate, cluttered, or ugly. Choose at least one. Sometimes choose all three."
                }
                p {
                    "There is a running joke that documentation is out-of-date the moment it is written. So the only way to keep it up to date is to "
                    span { class: "line-through", "write it again" }
                    " generate it from code."
                }
                p {
                    "For diagrams, the first can be solved by code calling a diagram generation library. The second needs a good layout algorithm, and a way of specifying the level of detail to display, and showing/hiding elements as they are relevant. The third requires the ability to style the elements."
                }
                p {
                    "The list of requirements I collated for a diagram generation library is:"
                }
                ol {
                    class: "list-decimal pl-5",
                    li {
                        span { class: "font-bold", "Auto layout:" }
                        " Remove the need to specify coordinates."
                    }
                    li {
                        span { class: "font-bold", "Stable layout:" }
                        " Elements don't jump around unpredictably as the diagram evolves."
                    }
                    li {
                        span { class: "font-bold", "Not node.js:" }
                        " JS/TS is a nightmare to maintain."
                    }
                    li {
                        span { class: "font-bold", "Compiles to WASM:" }
                        " Works in the browser."
                    }
                    li {
                        span { class: "font-bold", "Vector graphics:" }
                        " No pixelation when zoomed in."
                    }
                    li {
                        span { class: "font-bold", "Stylable:" }
                        " Otherwise no one wants to look at it."
                    }
                    li {
                        span { class: "font-bold", "Portable:" }
                        " Uploadable to external platforms without quality loss."
                    }
                    li {
                        span { class: "font-bold", "Editable:" }
                        " Diagrams can be experimented with by hand before codifying what to generate."
                    }
                    li {
                        span { class: "font-bold", "Text Format:" }
                        " Should be easy to serialize to / deserialize from the encoded format."
                    }
                }
                p {
                    "There are likely others, but these constraints were already a sufficiently tall order."
                }
            }
            Section {
                title: "Taking GraphViz To Its Limits",
                p {
                    "My first go at this was "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://azriel.im/dot_ix",
                        new_tab: true,
                        code {
                            "dot_ix"
                        }
                    }
                    ", where I created a simplified input model, fed that to "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://graphviz.org",
                        new_tab: true,
                        "GraphViz"
                    }
                    " to generate the SVG, then mashed "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://tailwindcss.com",
                        new_tab: true,
                        "TailWind CSS"
                    }
                    " classes onto the SVG with string replacements. This was perhaps the most fruitful attempt at discovering what I needed in a diagram generation tool."
                }
                p {
                    "GraphViz's ability to take in text input, layout diagrams, and output SVG was a great start, especially since it was also distributed as a WASM binary. Tailwind CSS classes improved the stylability of the SVG. "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://leptos.dev",
                        new_tab: true,
                        "Leptos"
                    }
                    " made building a UI in Rust far more tolerable than React, and JavaScript ("
                    span { class: "italic", "*shudders*" }
                    ") was used to glue things together."
                }
                p {
                    "The diagrams were delivering on the clarity I was imagining. I even made them "
                    span { class: "italic", "interactive" }
                    " with no scripts ("
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://peace.mk/book/learning_material/peace_zero_stress_automation/morale/aesthetics_outcome/web_example.svg",
                        new_tab: true,
                        "example"
                    }
                    ")."
                }
                p {
                    "However! "
                    code { "dot_ix" }
                    " was a side-side project that was becoming too hacky, as I trawled through GraphViz's (brilliant) documentation to convince GraphViz to generate diagrams to fit a use case it wasn't designed for."
                }
                p {
                    "After mashing a conglomerate of reusable parts together, I decided it's time to build my own."
                }
            }
            Section {
                title: "Rewriting it in Rust",
                p {
                    "I never intended to write a graph drawing library, but sometimes it's just one step towards a desired goal. Also, with the experience from "
                    code { "dot_ix" }
                    ", there were a other constraints that I wanted to solve, e.g. diagrams that work in both light and dark mode, or interactive animations."
                }
                p {
                    "I spent around 4 weekends "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://peace.mk/book/side_projects/disposition.html",
                        new_tab: true,
                        "designing "
                        code { "disposition" }
                    }
                    ", writing down potential features and issues, going through libraries and code to evaluate, \"is this a good library?\"."
                }
                p {
                    "When it came to building, I also switched from "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://leptos.dev",
                        new_tab: true,
                        "Leptos"
                    }
                    " to "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://dioxuslabs.com",
                        new_tab: true,
                        "Dioxus"
                    }
                    ". I really like Leptos' fine-grained reactivity model, yet Dioxus' syntax plays a lot nicer with my text editor's formatter, not to mention its CLI tool's UX being quite elegant."
                }
                p {
                    "For positioning elements in the diagram, figuring out how to layout nodes and edges was something I dreaded -- this was a substantial reason for using GraphViz in the first place."
                }
                ol {
                    class: "list-decimal pl-5",
                    li {
                        "Initially I used an approach of calculating node coordinates in a flex layout using "
                        Link {
                            class: "text-blue-400 hover:text-blue-300",
                            to: "https://github.com/DioxusLabs/taffy",
                            new_tab: true,
                            code { "taffy" }
                        }
                        "."
                    }
                    li {
                        "Then edges were placed by calculating the a Bezier curve between the closest faces of related nodes using "
                        Link {
                            class: "text-blue-400 hover:text-blue-300",
                            to: "https://github.com/linebender/kurbo",
                            new_tab: true,
                            code { "kurbo" }
                        }
                        ". I knew about Bezier curves from my university days, and it was serendipitous to have them come in useful again."
                    }
                    li {
                        "However! GraphViz isn't complex for nothing. Edges that cross nodes can block text, and that subtracts from the clarity of the diagram."
                    }
                    li {
                        "By chance, I had read a "
                        Link {
                            class: "text-blue-400 hover:text-blue-300",
                            to: "https://spidermonkey.dev/blog/2025/10/28/iongraph-web.html",
                            new_tab: true,
                            "SpiderMonkey blog post"
                        }
                        " 4 months prior, that said \"add a placeholder node for edges next to nodes\", and route your edges through the placeholder. At least that's what I think it said, because that's what I implemented."
                    }
                    li {
                        "The "
                        span { class: "italic", "sheer" }
                        " number of parameters to get edge paths to not overlap, is complex. I haven't got a perfect solution, but taking into account every edge that crosses a given rank from unrelated nodes of different nesting levels, to calculate "
                        span { class: "italic", "this" }
                        " edge's protrusion, is something I don't normally think about day-to-day."
                    }
                    li {
                        "Interactivity and animations are done with "
                        span { class: "italic", "clever" }
                        " use of CSS. By tagging each SVG element with an ID, and activating other CSS classes when a particular ID is focused, then when one element is focused, other elements can be made visible and animated."
                    }
                    li {
                        "Because Tailwind CSS classes are generated dynamically, "
                        Link {
                            class: "text-blue-400 hover:text-blue-300",
                            to: "https://gitlab.com/encre-org/encre-css",
                            new_tab: true,
                            code { "encre-css" }
                        }
                        " (RIIR-ed Tailwind CSS) is used to generate the "
                        span { class: "italic", "CSS" }
                        " at runtime."
                    }
                }
                p {
                    "There are still missing features as mentioned on the "
                    Link {
                        class: "text-blue-400 hover:text-blue-300",
                        to: "https://github.com/azriel91/disposition/blob/main/README.md",
                        new_tab: true,
                        code { "README.md" }
                    }
                    ", but I think it's useful as it stands."
                }
            }
            Section {
                title: "AI Disclaimer",
                p {
                    "This post is completely handwritten; "
                    code { "disposition" }
                    " is not."
                }
                p {
                    code { "disposition" }
                    " is the first project I've used an LLM to write. My take is that, LLMs are a very good pattern replicator, but if you give it freedom to find a solution, it will produce code that \"came from the masses\" -- and I find such code suboptimal in different ways (understandability, maintainability, ..)."
                }
                p {
                    "LLMs cannot tell if one library or another, or neither is suitable for a project. They cannot tell why drawing arrows between rectangular boxes is meaningful. Or why weaving together a flex layout, bezier curves, and CSS, "
                    span { class: "italic", "in this particular way" }
                    " is aesthetically pleasing. How too much detail is overwhelming, and how too little demands more."
                }
                p {
                    "There are certainly many tasks where my opinion is, \"If you are going to think of how to engineer that context into the LLM, why not think of the solution?\""
                }
                p {
                    "Yet I do not deny that with sufficient coercion, an LLM can begin to produce code that begins to mimic my own. But to devalue one's ability to design or create, and rely extensively on technology, is rejection of the very struggle that makes any activity worthwhile."
                }
                p {
                    "I've decided, I own my thoughts and control LLMs; not the other way around."
                }
            }
            Section {
                title: "Ending Note",
                p {
                    "Can we have nice things? I think so. Though often the effort to get there is "
                    span {
                        class: "italic",
                        "much, much"
                    }
                    " further than we think."
                }
                p {
                    code { "disposition" }
                    " is a side-side project, and sometimes I ponder what that difference between something free, and something that raises millions, is."
                }
                p {
                    "This was created on the shoulders of giants. Be free to use it, and add to it."
                }
            }
            small {
                class: "text-sm",
                "p/s: There is no CLI tool yet. The "
                Link {
                    class: "text-blue-400 hover:text-blue-300",
                    to: "/",
                    new_tab: true,
                    "playground"
                }
                " works offline once the page loads, and you can safely share the URL or YAML in the Text tab."
            }
        }
    }
}

#[component]
pub fn Section(title: &'static str, children: Element) -> Element {
    let id = title.to_ascii_lowercase().replace(" ", "-");
    rsx! {
        div {
            Link {
                class: "hover:underline",
                to: "#{id}",
                id,
                h3 {
                    class: "text-xl font-bold",
                    "{title}"
                }
            }
            {children}
        }
    }
}
