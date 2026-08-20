import os
import re
from pathlib import Path
import json

CONTENT_DIR = Path("content")
INDEX_FILE = Path("index.html")
SRC_DIR = Path("..")  # Search parent directory for Rust source files

def parse_rust_comments(root_dir):
    """
    Scans all .rs files in root_dir for @export directives and following block comments.
    Pattern matches:
      // @export path/to/page#section
      /*
      doc content
      */
    """
    docs_data = {}
    
    # Regex matches '// @export <path>' followed by optional whitespace and '/* <content> */'
    export_pattern = re.compile(
        r'//\s*@export\s+([^\n\r]+)[\r\n]+\s*/\*(.*?)\*/',
        re.DOTALL
    )

    resolved_root = root_dir.resolve()
    print(f" Scanning directory: {resolved_root}")

    files_scanned = 0
    exports_found = 0

    for file_path in root_dir.rglob("*.rs"):
        files_scanned += 1
        print(f" Checking: {file_path}")

        try:
            content = file_path.read_text(encoding="utf-8")
        except Exception as e:
            print(f"   [Error] Could not read file {file_path}: {e}")
            continue

        matches = list(export_pattern.finditer(content))
        if not matches:
            print("   └─ No @export comments found.")
            continue

        for match in matches:
            exports_found += 1
            export_path = match.group(1).strip()
            comment_body = match.group(2).strip()

            print(f"   ├──  Found @export: {export_path}")

            # 1. Temporarily extract <pre>...</pre> blocks to preserve literal newlines
            pre_blocks = []
            def save_pre(match_obj):
                pre_blocks.append(match_obj.group(0))
                return f"___PRE_BLOCK_{len(pre_blocks) - 1}___"

            temp_body = re.sub(r'<pre.*?>.*?</pre>', save_pre, comment_body, flags=re.DOTALL)

            # 2. Format regular paragraph text into HTML markup
            paragraphs = [p.strip() for p in temp_body.split("\n\n") if p.strip()]
            formatted_chunks = []

            for block in paragraphs:
                if "___PRE_BLOCK_" in block:
                    # Keep placeholders clean without wrapping in paragraph or <br> tags
                    formatted_chunks.append(block)
                else:
                    # Convert single newlines to <br> tags in normal paragraphs
                    formatted_chunks.append(f"<p>{block.replace('\n', '<br>')}</p>")

            formatted_body = "\n".join(formatted_chunks)

            # 3. Restore original intact <pre> blocks
            for i, pre in enumerate(pre_blocks):
                formatted_body = formatted_body.replace(f"___PRE_BLOCK_{i}___", pre)

            # Parse path structure (e.g., 'lib/io#file#file.getline()')
            parts = export_path.split("#", 1)
            raw_path = parts[0].strip()  # e.g., 'lib/io'
            anchor = parts[1].strip() if len(parts) > 1 else ""  # e.g., 'file.getline()'

            # Derive category and slug
            path_segments = [p for p in raw_path.split("/") if p]
            category = path_segments[0].capitalize() if path_segments else "General"
            
            # Create a URL-safe slug for the page
            slug = "-".join(path_segments).lower() or "index"
            title = path_segments[-1].capitalize() if path_segments else "Home"

            # Initialize structures in docs_data
            if category not in docs_data:
                docs_data[category] = {}

            if slug not in docs_data[category]:
                docs_data[category][slug] = {
                    "title": title,
                    "sections": []
                }

            # Add doc section entry
            docs_data[category][slug]["sections"].append({
                "anchor": anchor,
                "body": formatted_body
            })

    print(f"\n Scan Summary: Scanned {files_scanned} .rs files | Found {exports_found} @export doc blocks.\n")

    # Build final HTML body string per page with Table of Contents & <h2> headers
    processed_docs = {}
    for cat, pages in docs_data.items():
        processed_docs[cat] = {}
        for slug, data in pages.items():
            content_html = f"<h1>{data['title']}</h1>\n"
            
            # Collect sections with anchors to generate Table of Contents
            anchored_sections = [sec for sec in data["sections"] if sec["anchor"]]

            if len(anchored_sections) > 1:
                content_html += '<div class="table-of-contents">\n'
                content_html += '  <h2>Table of Contents</h2>\n'
                content_html += '  <ul>\n'
                for sec in anchored_sections:
                    anchor_id = sec["anchor"].replace("#", "-").replace(".", "-").replace("()", "")
                    content_html += f'    <li><a href="#{slug}#{anchor_id}"><code>{sec["anchor"]}</code></a></li>\n'
                content_html += '  </ul>\n'
                content_html += '</div>\n<hr>\n'

            # Build document sections
            for sec in data["sections"]:
                if sec["anchor"]:
                    anchor_id = sec["anchor"].replace("#", "-").replace(".", "-").replace("()", "")
                    content_html += f'<div id="{anchor_id}" class="doc-section">\n'
                    content_html += f'  <h2><code>{sec["anchor"]}</code></h2>\n'
                    content_html += f'  {sec["body"]}\n'
                    content_html += '</div>\n'
                else:
                    content_html += f'<div class="doc-section">\n  {sec["body"]}\n</div>\n'
            
            processed_docs[cat][slug] = {
                "title": data["title"],
                "body": content_html
            }

    return processed_docs

def get_first_page_slug(data):
    for _, pages in data.items():
        for slug, _ in pages.items():
            return slug
    return "index"

def generate_content_files(data):
    CONTENT_DIR.mkdir(exist_ok=True)
    if not data:
        print(" [Warning] No data found to generate content files!")
        return

    print(" Writing HTML files to content/")
    first_slug = get_first_page_slug(data)

    for category, pages in data.items():
        for slug, info in pages.items():
            file_path = CONTENT_DIR / f"{slug}.html"
            with open(file_path, "w", encoding="utf-8") as f:
                f.write(f"<!-- Auto-generated -->\n<section>\n{info['body']}\n</section>")
            print(f"   ├── Wrote: {file_path}")

    index_file = CONTENT_DIR / "index.html"
    if not index_file.exists():
        first_page = data[next(iter(data))][first_slug]
        with open(index_file, "w", encoding="utf-8") as f:
            f.write(f"<!-- Auto-generated -->\n<section>\n{first_page['body']}\n</section>")
        print(f"   ├── Wrote: {index_file} (default landing page)")

def generate_js_bundle(data):
    """
    Bundles all generated page bodies into a global window.DOCS_CONTENT object.
    """
    docs_map = {}
    for category, pages in data.items():
        for slug, info in pages.items():
            docs_map[slug] = f"<section>\n{info['body']}\n</section>"

    js_content = f"window.DOCS_CONTENT = {json.dumps(docs_map, indent=2)};"
    
    with open("docs-content.js", "w", encoding="utf-8") as f:
        f.write(js_content)
    print(" Wrote docs-content.js bundle")

def build_nav_html(data):
    html = ["<ul>"]
    for category, pages in data.items():
        if category.lower() in ["general", "index"]:
            for slug, info in pages.items():
                html.append(f'  <li><a href="#{slug}" data-page="{slug}">{info["title"]}</a></li>')
        else:
            html.append('  <li class="nav-dropdown">')
            html.append('    <details>')
            html.append(f'      <summary>{category}</summary>')
            html.append('      <ul>')
            for slug, info in pages.items():
                html.append(f'        <li><a href="#{slug}" data-page="{slug}">{info["title"]}</a></li>')
            html.append('      </ul>')
            html.append('    </details>')
            html.append('  </li>')
    html.append("</ul>")
    return "\n".join(html)

def update_index_nav(nav_html):
    if not INDEX_FILE.exists():
        print(f" [Error] {INDEX_FILE} not found! Navigation was not updated.")
        return

    with open(INDEX_FILE, "r", encoding="utf-8") as f:
        content = f.read()

    pattern = r"(<!-- NAV_START -->)(.*?)(<!-- NAV_END -->)"
    if not re.search(pattern, content, flags=re.DOTALL):
        print(f" [Error] <!-- NAV_START --> and <!-- NAV_END --> markers missing in {INDEX_FILE}!")
        return

    replacement = f"\\1\n{nav_html}\n\\3"
    updated_content = re.sub(pattern, replacement, content, flags=re.DOTALL)

    with open(INDEX_FILE, "w", encoding="utf-8") as f:
        f.write(updated_content)

    print(f" Updated navigation inside {INDEX_FILE}")

if __name__ == "__main__":
    docs_data = parse_rust_comments(SRC_DIR)
    generate_js_bundle(docs_data)
    generate_content_files(docs_data)
    nav_markup = build_nav_html(docs_data)
    update_index_nav(nav_markup)
    print("\n Process completed!")