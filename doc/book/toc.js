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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="first-prompt.html"><strong aria-hidden="true">3.</strong> Your First Prompt</a></li><li class="chapter-item expanded affix "><li class="part-title">User Guide</li><li class="chapter-item expanded "><a href="creating-prompts.html"><strong aria-hidden="true">4.</strong> Creating Prompts</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="yaml-front-matter.html"><strong aria-hidden="true">4.1.</strong> YAML Front Matter</a></li><li class="chapter-item expanded "><a href="template-variables.html"><strong aria-hidden="true">4.2.</strong> Template Variables</a></li><li class="chapter-item expanded "><a href="custom-filters.html"><strong aria-hidden="true">4.3.</strong> Custom Filters</a></li><li class="chapter-item expanded "><a href="prompt-organization.html"><strong aria-hidden="true">4.4.</strong> Prompt Organization</a></li></ol></li><li class="chapter-item expanded "><a href="claude-code-integration.html"><strong aria-hidden="true">5.</strong> Using with Claude Code</a></li><li class="chapter-item expanded "><a href="cli-reference.html"><strong aria-hidden="true">6.</strong> Command Line Interface</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="cli-serve.html"><strong aria-hidden="true">6.1.</strong> serve</a></li><li class="chapter-item expanded "><a href="cli-search.html"><strong aria-hidden="true">6.2.</strong> search</a></li><li class="chapter-item expanded "><a href="cli-test.html"><strong aria-hidden="true">6.3.</strong> test</a></li><li class="chapter-item expanded "><a href="cli-doctor.html"><strong aria-hidden="true">6.4.</strong> doctor</a></li><li class="chapter-item expanded "><a href="cli-completion.html"><strong aria-hidden="true">6.5.</strong> completion</a></li></ol></li><li class="chapter-item expanded "><li class="part-title">Library Usage</li><li class="chapter-item expanded "><a href="library-usage.html"><strong aria-hidden="true">7.</strong> Rust Library Guide</a></li><li class="chapter-item expanded "><a href="library-api.html"><strong aria-hidden="true">8.</strong> Library API Reference</a></li><li class="chapter-item expanded "><a href="library-examples.html"><strong aria-hidden="true">9.</strong> Integration Examples</a></li><li class="chapter-item expanded "><a href="api/swissarmyhammer/index.html"><strong aria-hidden="true">10.</strong> 🔗 Rustdoc API Documentation</a></li><li class="chapter-item expanded "><a href="https://docs.rs/swissarmyhammer.html"><strong aria-hidden="true">11.</strong> 📚 docs.rs API Reference</a></li><li class="chapter-item expanded affix "><li class="part-title">Advanced Usage</li><li class="chapter-item expanded "><a href="advanced-prompts.html"><strong aria-hidden="true">12.</strong> Advanced Prompt Techniques</a></li><li class="chapter-item expanded "><a href="workflows.html"><strong aria-hidden="true">13.</strong> Workflows</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="workflow-examples.html"><strong aria-hidden="true">13.1.</strong> Workflow Examples</a></li><li class="chapter-item expanded "><a href="workflow-patterns.html"><strong aria-hidden="true">13.2.</strong> Workflow Patterns</a></li></ol></li><li class="chapter-item expanded "><a href="search-guide.html"><strong aria-hidden="true">14.</strong> Search and Discovery</a></li><li class="chapter-item expanded "><a href="testing-guide.html"><strong aria-hidden="true">15.</strong> Testing and Debugging</a></li><li class="chapter-item expanded "><a href="sharing-guide.html"><strong aria-hidden="true">16.</strong> Sharing and Collaboration</a></li><li class="chapter-item expanded "><a href="mcp-protocol.html"><strong aria-hidden="true">17.</strong> MCP Protocol</a></li><li class="chapter-item expanded "><a href="configuration.html"><strong aria-hidden="true">18.</strong> Configuration</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="configuration-schema.html"><strong aria-hidden="true">18.1.</strong> Configuration Schema Reference</a></li></ol></li><li class="chapter-item expanded "><a href="file-watching.html"><strong aria-hidden="true">19.</strong> File Watching</a></li><li class="chapter-item expanded "><a href="prompt-overrides.html"><strong aria-hidden="true">20.</strong> Prompt Overrides</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference</li><li class="chapter-item expanded "><a href="builtin-prompts.html"><strong aria-hidden="true">21.</strong> Built-in Prompts</a></li><li class="chapter-item expanded "><a href="filters-reference.html"><strong aria-hidden="true">22.</strong> Custom Filters Reference</a></li><li class="chapter-item expanded "><a href="examples.html"><strong aria-hidden="true">23.</strong> Examples</a></li><li class="chapter-item expanded "><a href="troubleshooting.html"><strong aria-hidden="true">24.</strong> Troubleshooting</a></li><li class="chapter-item expanded affix "><li class="part-title">Development</li><li class="chapter-item expanded "><a href="contributing.html"><strong aria-hidden="true">25.</strong> Contributing</a></li><li class="chapter-item expanded "><a href="development.html"><strong aria-hidden="true">26.</strong> Development Setup</a></li><li class="chapter-item expanded "><a href="testing.html"><strong aria-hidden="true">27.</strong> Testing</a></li><li class="chapter-item expanded "><a href="release-process.html"><strong aria-hidden="true">28.</strong> Release Process</a></li><li class="chapter-item expanded affix "><li class="part-title">Appendix</li><li class="chapter-item expanded "><a href="changelog.html"><strong aria-hidden="true">29.</strong> Changelog</a></li><li class="chapter-item expanded "><a href="license.html"><strong aria-hidden="true">30.</strong> License</a></li></ol>';
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
