// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="basic_usage.html"><strong aria-hidden="true">1.</strong> Basic Usage</a></li><li class="chapter-item expanded "><a href="yaml_syntax.html"><strong aria-hidden="true">2.</strong> Syntax</a></li><li class="chapter-item expanded "><a href="checks.html"><strong aria-hidden="true">3.</strong> Checks</a></li><li class="chapter-item expanded affix "><li class="part-title">Advanced Use-cases</li><li class="chapter-item expanded "><a href="adv_formatting.html"><strong aria-hidden="true">4.</strong> Formatting</a></li><li class="chapter-item expanded "><a href="adv_layout.html"><strong aria-hidden="true">5.</strong> Layout</a></li><li class="chapter-item expanded "><a href="adv_layers.html"><strong aria-hidden="true">6.</strong> Layers</a></li><li class="chapter-item expanded "><a href="adv_stylesheets.html"><strong aria-hidden="true">7.</strong> Stylesheets</a></li><li class="chapter-item expanded "><a href="adv_evidence.html"><strong aria-hidden="true">8.</strong> Evidence</a></li><li class="chapter-item expanded "><a href="adv_statistics.html"><strong aria-hidden="true">9.</strong> Statistics</a></li><li class="chapter-item expanded "><a href="adv_interfacing.html"><strong aria-hidden="true">10.</strong> Interfacing</a></li><li class="chapter-item expanded affix "><li class="part-title">Extensions</li><li class="chapter-item expanded "><a href="ext_mod.html"><strong aria-hidden="true">11.</strong> Modular Extension</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="ext_mod_distdev.html"><strong aria-hidden="true">11.1.</strong> Distributed Development</a></li><li class="chapter-item expanded "><a href="ext_mod_info.html"><strong aria-hidden="true">11.2.</strong> Module Info</a></li></ol></li><li class="chapter-item expanded "><a href="ext_confidence.html"><strong aria-hidden="true">12.</strong> Confidence Argument Extension</a></li><li class="chapter-item expanded "><a href="ext_dialectic.html"><strong aria-hidden="true">13.</strong> Dialectic Extension</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="standard_support.html"><strong aria-hidden="true">14.</strong> Standard Support</a></li><li class="chapter-item expanded "><a href="troubleshooting.html"><strong aria-hidden="true">15.</strong> Troubleshooting</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="design_goals.html"><strong aria-hidden="true">16.</strong> Design Goals</a></li><li class="chapter-item expanded "><a href="history.html"><strong aria-hidden="true">17.</strong> History</a></li><li class="chapter-item expanded "><a href="migration.html"><strong aria-hidden="true">18.</strong> Migration between Different Versions</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
