import os


changelog_path = os.path.join(os.path.dirname(__file__), "..", "CHANGELOG.md")

with open(changelog_path, "r") as file:
    changelog = file.read()

# remove the top-level header
changelog = changelog.split("# Changelog")[1].strip()
out = "<releases>\n"
# each release has a second-level header
for release in changelog.split("\n## ")[1:]:
    # only the header has a double newline after it
    header = release.split("\n\n")[0].strip()
    version, date, *_ = header.split(" ")
    date = date.strip("()")

    out += f'<release version="{version}" date="{date}">\n'

    # parse sections like "Added", "Changed", etc. which have a third-level header
    for section in release.split("\n### ")[1:]:
        subheader, *content = section.split("\n\n")
        out += f"<p>{subheader.strip()}</p>\n<ul>\n"

        # put each change in a list item
        for line in "".join(content).split("\n"):
            line = line.replace("- ", "").strip()
            # escape XML special characters
            line = line.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")
            if line:
                out += f"<li>{line}</li>\n"

        out += "</ul>\n"

    out += "</release>\n"

out += "</releases>"

# replace the <releases> section in the flathub metainfo.xml with the new one
metainfo_path = os.path.join(
    os.path.dirname(__file__), "..", "com.kesomannen.gale.metainfo.xml"
)

with open(metainfo_path, "r") as file:
    metainfo = file.read()

releases_start = metainfo.index("<releases>")
releases_end = metainfo.index("</releases>") + len("</releases>")
new_metainfo = metainfo[:releases_start] + out + metainfo[releases_end:]

with open(metainfo_path, "w") as file:
    file.write(new_metainfo)

print("Release notes updated in com.kesomannen.gale.metainfo.xml")
