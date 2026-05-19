import React from 'react';
import { ExternalLink } from 'lucide-react';
import type { ChainlinkCard, ChainlinkStatus } from './parser';

const PRIORITIES = ['critical', 'high', 'medium', 'low'];

export interface ChainlinkKanbanProps {
  cards: ChainlinkCard[];
  onOpenPage: (pageRef: string) => void;
}

export function ChainlinkKanban({ cards, onOpenPage }: ChainlinkKanbanProps) {
  const columns: ChainlinkStatus[] = ['open', 'closed'];

  return (
    <div className="chainlink-board" role="list">
      {columns.map((status) => {
        const columnCards = cards.filter((card) => card.status === status);
        return (
          <section className="chainlink-column" key={status} aria-label={`${status} issues`}>
            <header className="chainlink-column__header">
              <h3>{status === 'open' ? 'Open' : 'Closed'}</h3>
              <span>{columnCards.length}</span>
            </header>
            {PRIORITIES.map((priority) => {
              const priorityCards = columnCards.filter((card) => card.priority === priority);
              if (priorityCards.length === 0) {
                return null;
              }
              return (
                <div className="chainlink-priority" key={`${status}-${priority}`}>
                  <div className="chainlink-priority__label">{priority}</div>
                  {priorityCards.map((card) => (
                    <button
                      className="chainlink-card"
                      key={`${card.status}-${card.id}`}
                      onClick={() => onOpenPage(card.pageRef)}
                      type="button"
                    >
                      <span className="chainlink-card__title">
                        <span>#{card.id.toString().padStart(4, '0')}</span>
                        {card.title}
                      </span>
                      <span className="chainlink-card__meta">
                        <span>{card.time || '0m'}</span>
                        {card.labels.map((label) => (
                          <span className="chainlink-card__label" key={label}>{label}</span>
                        ))}
                        <ExternalLink aria-hidden="true" size={14} />
                      </span>
                    </button>
                  ))}
                </div>
              );
            })}
            {columnCards.length === 0 ? <p className="chainlink-empty">No issues</p> : null}
          </section>
        );
      })}
    </div>
  );
}
