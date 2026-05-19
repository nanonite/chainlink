# logseq-plugin-chainlink

A Logseq plugin that renders `chainlink___dashboard.md` as a kanban board inside Logseq.

## Usage

1. Export Chainlink pages with `chainlink logseq export`.
2. Install this plugin in Logseq desktop.
3. Set `Dashboard file path` to the generated `chainlink___dashboard.md` file.
4. Use the `/chainlink-dashboard` slash command on any page.

The plugin reads markdown only. It does not connect to Chainlink's database and does not start a server.
