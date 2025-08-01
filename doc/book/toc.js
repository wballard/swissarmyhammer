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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="first-prompt.html"><strong aria-hidden="true">3.</strong> Your First Prompt</a></li><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="creating-prompts.html"><strong aria-hidden="true">4.</strong> Creating Prompts</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="yaml-front-matter.html"><strong aria-hidden="true">4.1.</strong> YAML Front Matter</a></li><li class="chapter-item expanded "><a href="template-variables.html"><strong aria-hidden="true">4.2.</strong> Template Variables</a></li><li class="chapter-item expanded "><a href="custom-filters.html"><strong aria-hidden="true">4.3.</strong> Custom Filters</a></li><li class="chapter-item expanded "><a href="prompt-organization.html"><strong aria-hidden="true">4.4.</strong> Prompt Organization</a></li></ol></li><li class="chapter-item expanded "><a href="claude-code-integration.html"><strong aria-hidden="true">5.</strong> Using with Claude Code</a></li><li class="chapter-item expanded "><a href="cli-reference.html"><strong aria-hidden="true">6.</strong> Command Line Interface</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="cli-serve.html"><strong aria-hidden="true">6.1.</strong> serve</a></li><li class="chapter-item expanded "><a href="cli-search.html"><strong aria-hidden="true">6.2.</strong> search</a></li><li class="chapter-item expanded "><a href="cli-test.html"><strong aria-hidden="true">6.3.</strong> test</a></li><li class="chapter-item expanded "><a href="cli-doctor.html"><strong aria-hidden="true">6.4.</strong> doctor</a></li><li class="chapter-item expanded "><a href="cli-completion.html"><strong aria-hidden="true">6.5.</strong> completion</a></li><li class="chapter-item expanded "><a href="cli-memoranda.html"><strong aria-hidden="true">6.6.</strong> memoranda</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Advanced Usage</li><li class="chapter-item expanded "><a href="advanced-prompts.html"><strong aria-hidden="true">7.</strong> Advanced Prompt Techniques</a></li><li class="chapter-item expanded "><a href="issue-management.html"><strong aria-hidden="true">8.</strong> Issue Management</a></li><li class="chapter-item expanded "><a href="workflows.html"><strong aria-hidden="true">9.</strong> Workflows</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="workflow-examples.html"><strong aria-hidden="true">9.1.</strong> Workflow Examples</a></li><li class="chapter-item expanded "><a href="workflow-patterns.html"><strong aria-hidden="true">9.2.</strong> Workflow Patterns</a></li></ol></li><li class="chapter-item expanded "><a href="search-guide.html"><strong aria-hidden="true">10.</strong> Search and Discovery</a></li><li class="chapter-item expanded "><a href="search-architecture.html"><strong aria-hidden="true">11.</strong> Search Architecture</a></li><li class="chapter-item expanded "><a href="index-management.html"><strong aria-hidden="true">12.</strong> Index Management</a></li><li class="chapter-item expanded "><a href="testing-guide.html"><strong aria-hidden="true">13.</strong> Testing and Debugging</a></li><li class="chapter-item expanded "><a href="configuration.html"><strong aria-hidden="true">14.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="prompt-overrides.html"><strong aria-hidden="true">15.</strong> Prompt Overrides</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference</li><li class="chapter-item expanded "><a href="builtin-prompts.html"><strong aria-hidden="true">16.</strong> Built-in Prompts</a></li><li class="chapter-item expanded "><a href="examples.html"><strong aria-hidden="true">17.</strong> Examples</a></li><li class="chapter-item expanded "><a href="troubleshooting.html"><strong aria-hidden="true">18.</strong> Troubleshooting</a></li></ol>';
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
