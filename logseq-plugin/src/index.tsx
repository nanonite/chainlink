import '@logseq/libs';
import React from 'react';
import { createRoot } from 'react-dom/client';
import { ChainlinkKanban } from './kanban';
import { parseDashboard } from './parser';
import { styles } from './styles';

declare global {
  interface Window {
    require?: (moduleName: string) => { readFileSync: (path: string, encoding: string) => string };
  }
}

const SETTINGS_SCHEMA = [
  {
    key: 'dashboardPath',
    type: 'string',
    title: 'Dashboard file path',
    description: 'Absolute path to chainlink___dashboard.md',
    default: ''
  }
] as const;

async function readDashboard(path: string): Promise<string> {
  if (!path.trim()) {
    throw new Error('Dashboard file path is not configured');
  }

  if (window.require) {
    const fs = window.require('fs');
    return fs.readFileSync(path, 'utf8');
  }

  const response = await fetch(path);
  if (!response.ok) {
    throw new Error(`Unable to read dashboard file: ${response.status}`);
  }
  return response.text();
}

async function renderDashboard(slot: string) {
  const container = parent.document.getElementById(slot);
  if (!container) {
    return;
  }

  const dashboardPath = String(logseq.settings?.dashboardPath ?? '');
  try {
    const markdown = await readDashboard(dashboardPath);
    const cards = parseDashboard(markdown);
    const root = createRoot(container);
    root.render(
      <ChainlinkKanban
        cards={cards}
        onOpenPage={(pageRef) => void logseq.App.pushState('page', { name: pageRef })}
      />
    );
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    container.innerHTML = `<div class="chainlink-empty">${escapeHtml(message)}</div>`;
  }
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function main() {
  logseq.useSettingsSchema(SETTINGS_SCHEMA as unknown as Parameters<typeof logseq.useSettingsSchema>[0]);
  logseq.provideStyle(styles);
  logseq.Editor.registerSlashCommand('chainlink-dashboard', async () => {
    await logseq.Editor.insertAtEditingCursor('{{renderer :chainlink-dashboard}}');
  });
  logseq.App.onMacroRendererSlotted(({ slot, payload }) => {
    if (payload.arguments?.[0] === ':chainlink-dashboard') {
      void renderDashboard(slot);
    }
  });
}

logseq.ready(main).catch(console.error);
