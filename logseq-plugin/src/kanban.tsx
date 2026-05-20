import React from 'react';
import { CheckCircle2, Clock3, ExternalLink, Flame, ListChecks } from 'lucide-react';
import type { ChainlinkCard, ChainlinkStatus } from './parser';

const PRIORITIES = ['critical', 'high', 'medium', 'low'];
const PRIORITY_LABELS: Record<string, string> = {
  critical: 'Critical',
  high: 'High',
  medium: 'Medium',
  low: 'Low'
};

export interface ChainlinkKanbanProps {
  cards: ChainlinkCard[];
  onOpenPage: (pageRef: string) => void;
}

export function ChainlinkKanban({ cards, onOpenPage }: ChainlinkKanbanProps) {
  const openCards = cards.filter((card) => card.status === 'open');
  const closedCards = cards.filter((card) => card.status === 'closed');
  const highSignal = openCards.filter((card) => card.priority === 'critical' || card.priority === 'high').length;
  const timedCards = openCards.filter((card) => card.time && card.time !== '0m').length;

  return (
    <div className="chainlink-shell">
      <header className="chainlink-summary">
        <div className="chainlink-summary__item chainlink-summary__item--open">
          <ListChecks aria-hidden="true" size={18} />
          <span>Open</span>
          <strong>{openCards.length}</strong>
        </div>
        <div className="chainlink-summary__item chainlink-summary__item--risk">
          <Flame aria-hidden="true" size={18} />
          <span>High signal</span>
          <strong>{highSignal}</strong>
        </div>
        <div className="chainlink-summary__item chainlink-summary__item--time">
          <Clock3 aria-hidden="true" size={18} />
          <span>Timed</span>
          <strong>{timedCards}</strong>
        </div>
        <div className="chainlink-summary__item chainlink-summary__item--done">
          <CheckCircle2 aria-hidden="true" size={18} />
          <span>Recent wins</span>
          <strong>{closedCards.length}</strong>
        </div>
      </header>
      <div className="chainlink-board" role="list">
        <IssueColumn status="open" cards={openCards} onOpenPage={onOpenPage} />
        <IssueColumn status="closed" cards={closedCards} onOpenPage={onOpenPage} />
      </div>
    </div>
  );
}

function IssueColumn({ status, cards, onOpenPage }: { status: ChainlinkStatus; cards: ChainlinkCard[]; onOpenPage: (pageRef: string) => void }) {
  return (
    <section className="chainlink-column" aria-label={`${status} issues`}>
      <header className="chainlink-column__header">
        <h3>{status === 'open' ? 'Open Work' : 'Recently Closed'}</h3>
        <span>{cards.length}</span>
      </header>
      {PRIORITIES.map((priority) => {
        const priorityCards = cards.filter((card) => card.priority === priority);
        if (priorityCards.length === 0) {
          return null;
        }
        return (
          <div className="chainlink-priority" key={`${status}-${priority}`}>
            <div className={`chainlink-priority__label chainlink-priority__label--${priority}`}>
              {PRIORITY_LABELS[priority] ?? priority}
            </div>
            {priorityCards.map((card) => (
              <IssueCard card={card} key={`${card.status}-${card.id}`} onOpenPage={onOpenPage} />
            ))}
          </div>
        );
      })}
      {cards.length === 0 ? <p className="chainlink-empty">No issues in this lane</p> : null}
    </section>
  );
}

function IssueCard({ card, onOpenPage }: { card: ChainlinkCard; onOpenPage: (pageRef: string) => void }) {
  return (
    <button className="chainlink-card" onClick={() => onOpenPage(card.pageRef)} type="button">
      <span className="chainlink-card__title">
        <span>#{card.id.toString().padStart(4, '0')}</span>
        {card.title}
      </span>
      <span className="chainlink-card__meta">
        <span className="chainlink-card__time">{card.time || '0m'}</span>
        {card.labels.map((label) => (
          <span className="chainlink-card__label" key={label}>{label}</span>
        ))}
        <ExternalLink aria-hidden="true" size={14} />
      </span>
    </button>
  );
}
