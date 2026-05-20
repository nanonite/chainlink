export const styles = `
.chainlink-shell {
  display: grid;
  gap: 12px;
  width: min(100%, 980px);
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

.chainlink-summary {
  display: grid;
  gap: 8px;
  grid-template-columns: repeat(4, minmax(120px, 1fr));
}

.chainlink-summary__item {
  align-items: center;
  background: var(--ls-primary-background-color, #ffffff);
  border: 1px solid var(--ls-border-color, #d8dee4);
  border-radius: 8px;
  display: grid;
  gap: 2px 8px;
  grid-template-columns: auto 1fr;
  min-width: 0;
  padding: 10px;
}

.chainlink-summary__item svg {
  grid-row: span 2;
}

.chainlink-summary__item span {
  color: var(--ls-secondary-text-color, #57606a);
  font-size: 11px;
  line-height: 1.2;
}

.chainlink-summary__item strong {
  color: var(--ls-primary-text-color, #24292f);
  font-size: 18px;
  line-height: 1.1;
}

.chainlink-summary__item--open svg { color: #0969da; }
.chainlink-summary__item--risk svg { color: #cf222e; }
.chainlink-summary__item--time svg { color: #8250df; }
.chainlink-summary__item--done svg { color: #1a7f37; }

.chainlink-board {
  display: grid;
  gap: 12px;
  grid-template-columns: minmax(280px, 1.4fr) minmax(240px, 1fr);
}

.chainlink-column {
  background: var(--ls-primary-background-color, #ffffff);
  border: 1px solid var(--ls-border-color, #d8dee4);
  border-radius: 8px;
  min-width: 0;
  padding: 10px;
}

.chainlink-column__header {
  align-items: center;
  border-bottom: 1px solid var(--ls-border-color, #d8dee4);
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
  padding-bottom: 8px;
}

.chainlink-column__header h3 {
  font-size: 14px;
  font-weight: 700;
  line-height: 1.2;
  margin: 0;
}

.chainlink-column__header span {
  color: var(--ls-secondary-text-color, #57606a);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}

.chainlink-priority + .chainlink-priority {
  margin-top: 12px;
}

.chainlink-priority__label {
  align-items: center;
  color: var(--ls-secondary-text-color, #57606a);
  display: flex;
  font-size: 11px;
  font-weight: 800;
  gap: 6px;
  letter-spacing: 0;
  margin-bottom: 6px;
  text-transform: uppercase;
}

.chainlink-priority__label::before {
  border-radius: 999px;
  content: "";
  display: block;
  height: 8px;
  width: 8px;
}

.chainlink-priority__label--critical::before { background: #cf222e; }
.chainlink-priority__label--high::before { background: #fb8500; }
.chainlink-priority__label--medium::before { background: #0969da; }
.chainlink-priority__label--low::before { background: #1a7f37; }

.chainlink-card {
  appearance: none;
  background: var(--ls-secondary-background-color, #f6f8fa);
  border: 1px solid transparent;
  border-radius: 7px;
  color: var(--ls-primary-text-color, #24292f);
  cursor: pointer;
  display: block;
  font: inherit;
  margin: 6px 0 0;
  padding: 10px;
  text-align: left;
  transition: border-color 120ms ease, background 120ms ease;
  width: 100%;
}

.chainlink-card:hover,
.chainlink-card:focus-visible {
  background: var(--ls-primary-background-color, #ffffff);
  border-color: var(--ls-link-text-color, #0969da);
  outline: none;
}

.chainlink-card__title {
  display: grid;
  gap: 8px;
  grid-template-columns: auto 1fr;
  font-size: 13px;
  font-weight: 650;
  line-height: 1.35;
  overflow-wrap: anywhere;
}

.chainlink-card__title span {
  color: var(--ls-secondary-text-color, #57606a);
  font-variant-numeric: tabular-nums;
}

.chainlink-card__meta {
  align-items: center;
  color: var(--ls-secondary-text-color, #57606a);
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 9px;
  min-height: 18px;
}

.chainlink-card__meta,
.chainlink-card__label,
.chainlink-card__time {
  font-size: 11px;
}

.chainlink-card__time,
.chainlink-card__label {
  background: var(--ls-tertiary-background-color, #eaeef2);
  border-radius: 999px;
  padding: 2px 7px;
}

.chainlink-card__time {
  color: var(--ls-primary-text-color, #24292f);
}

.chainlink-empty {
  color: var(--ls-secondary-text-color, #57606a);
  font-size: 12px;
  margin: 8px 0 0;
}

@media (max-width: 760px) {
  .chainlink-summary,
  .chainlink-board {
    grid-template-columns: 1fr;
  }
}
`;
