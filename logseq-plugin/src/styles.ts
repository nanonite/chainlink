export const styles = `
.chainlink-board {
  display: grid;
  gap: 12px;
  grid-template-columns: repeat(2, minmax(220px, 1fr));
  width: min(100%, 920px);
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

.chainlink-column {
  border: 1px solid var(--ls-border-color, #d8dee4);
  border-radius: 8px;
  background: var(--ls-primary-background-color, #ffffff);
  min-width: 0;
  padding: 10px;
}

.chainlink-column__header {
  align-items: center;
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}

.chainlink-column__header h3 {
  font-size: 14px;
  font-weight: 650;
  line-height: 1.2;
  margin: 0;
  text-transform: capitalize;
}

.chainlink-column__header span {
  color: var(--ls-secondary-text-color, #57606a);
  font-size: 12px;
}

.chainlink-priority + .chainlink-priority {
  margin-top: 12px;
}

.chainlink-priority__label {
  color: var(--ls-secondary-text-color, #57606a);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0;
  margin-bottom: 6px;
  text-transform: uppercase;
}

.chainlink-card {
  appearance: none;
  background: var(--ls-secondary-background-color, #f6f8fa);
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--ls-primary-text-color, #24292f);
  cursor: pointer;
  display: block;
  font: inherit;
  margin: 6px 0 0;
  padding: 9px;
  text-align: left;
  width: 100%;
}

.chainlink-card:hover,
.chainlink-card:focus-visible {
  border-color: var(--ls-link-text-color, #0969da);
  outline: none;
}

.chainlink-card__title {
  display: grid;
  gap: 6px;
  grid-template-columns: auto 1fr;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.3;
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
  margin-top: 8px;
  min-height: 18px;
}

.chainlink-card__meta,
.chainlink-card__label {
  font-size: 11px;
}

.chainlink-card__label {
  background: var(--ls-tertiary-background-color, #eaeef2);
  border-radius: 999px;
  padding: 2px 6px;
}

.chainlink-empty {
  color: var(--ls-secondary-text-color, #57606a);
  font-size: 12px;
  margin: 8px 0 0;
}

@media (max-width: 640px) {
  .chainlink-board {
    grid-template-columns: 1fr;
  }
}
`;
