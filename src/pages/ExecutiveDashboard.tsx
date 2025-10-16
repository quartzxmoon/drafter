// Executive Dashboard - Premium C-Level Interface
// Big Law aesthetic with real-time metrics and analytics

import React, { useState, useEffect } from 'react';
import { designSystem } from '../styles/design-system';

// Executive Metric Cards
interface MetricCardProps {
  title: string;
  value: string;
  change: string;
  changeType: 'positive' | 'negative' | 'neutral';
  icon: string;
}

const MetricCard: React.FC<MetricCardProps> = ({ title, value, change, changeType, icon }) => {
  const changeColor = changeType === 'positive' ? designSystem.colors.success[500] :
                      changeType === 'negative' ? designSystem.colors.danger[500] :
                      designSystem.colors.neutral[500];

  return (
    <div style={{
      background: 'white',
      borderRadius: designSystem.borderRadius.lg,
      padding: designSystem.spacing[6],
      boxShadow: designSystem.shadows.md,
      transition: designSystem.transitions.base,
      cursor: 'pointer',
      ':hover': {
        boxShadow: designSystem.shadows.xl,
        transform: 'translateY(-4px)',
      }
    }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div>
          <p style={{
            fontSize: designSystem.typography.fontSize.sm,
            color: designSystem.colors.neutral[600],
            fontWeight: designSystem.typography.fontWeight.medium,
            marginBottom: designSystem.spacing[2],
            textTransform: 'uppercase',
            letterSpacing: designSystem.typography.letterSpacing.wide,
          }}>
            {title}
          </p>
          <h2 style={{
            fontSize: designSystem.typography.fontSize['4xl'],
            fontWeight: designSystem.typography.fontWeight.bold,
            color: designSystem.colors.neutral[900],
            marginBottom: designSystem.spacing[2],
            fontFamily: designSystem.typography.fontFamily.display,
          }}>
            {value}
          </h2>
          <p style={{
            fontSize: designSystem.typography.fontSize.sm,
            color: changeColor,
            fontWeight: designSystem.typography.fontWeight.semibold,
          }}>
            {change} vs last month
          </p>
        </div>
        <div style={{
          width: '56px',
          height: '56px',
          borderRadius: designSystem.borderRadius.lg,
          background: `linear-gradient(135deg, ${designSystem.colors.primary[500]} 0%, ${designSystem.colors.primary[700]} 100%)`,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: designSystem.typography.fontSize['2xl'],
        }}>
          {icon}
        </div>
      </div>
    </div>
  );
};

// Matter Pipeline Status
interface PipelineStage {
  name: string;
  count: number;
  value: number;
  color: string;
}

const PipelineView: React.FC = () => {
  const stages: PipelineStage[] = [
    { name: 'Intake', count: 12, value: 180000, color: designSystem.colors.info[500] },
    { name: 'Qualified', count: 8, value: 240000, color: designSystem.colors.primary[500] },
    { name: 'Retained', count: 15, value: 675000, color: designSystem.colors.gold[500] },
    { name: 'Active', count: 42, value: 1850000, color: designSystem.colors.success[500] },
    { name: 'Settlement', count: 7, value: 420000, color: designSystem.colors.gold[600] },
  ];

  return (
    <div style={{
      background: 'white',
      borderRadius: designSystem.borderRadius.lg,
      padding: designSystem.spacing[6],
      boxShadow: designSystem.shadows.md,
    }}>
      <h3 style={{
        fontSize: designSystem.typography.fontSize.xl,
        fontWeight: designSystem.typography.fontWeight.bold,
        color: designSystem.colors.neutral[900],
        marginBottom: designSystem.spacing[6],
        fontFamily: designSystem.typography.fontFamily.display,
      }}>
        Matter Pipeline
      </h3>

      <div style={{ display: 'flex', gap: designSystem.spacing[4], marginBottom: designSystem.spacing[6] }}>
        {stages.map((stage) => (
          <div key={stage.name} style={{ flex: 1 }}>
            <div style={{
              background: stage.color,
              height: '8px',
              borderRadius: designSystem.borderRadius.full,
              marginBottom: designSystem.spacing[3],
            }} />
            <p style={{
              fontSize: designSystem.typography.fontSize.xs,
              color: designSystem.colors.neutral[600],
              fontWeight: designSystem.typography.fontWeight.medium,
              marginBottom: designSystem.spacing[1],
            }}>
              {stage.name}
            </p>
            <p style={{
              fontSize: designSystem.typography.fontSize.xl,
              fontWeight: designSystem.typography.fontWeight.bold,
              color: designSystem.colors.neutral[900],
              marginBottom: designSystem.spacing[1],
            }}>
              {stage.count}
            </p>
            <p style={{
              fontSize: designSystem.typography.fontSize.sm,
              color: designSystem.colors.neutral[600],
            }}>
              ${(stage.value / 1000).toFixed(0)}K
            </p>
          </div>
        ))}
      </div>

      <div style={{
        borderTop: `1px solid ${designSystem.colors.neutral[200]}`,
        paddingTop: designSystem.spacing[4],
        display: 'flex',
        justifyContent: 'space-between',
      }}>
        <div>
          <p style={{
            fontSize: designSystem.typography.fontSize.sm,
            color: designSystem.colors.neutral[600],
            marginBottom: designSystem.spacing[1],
          }}>
            Total Pipeline Value
          </p>
          <p style={{
            fontSize: designSystem.typography.fontSize['2xl'],
            fontWeight: designSystem.typography.fontWeight.bold,
            color: designSystem.colors.neutral[900],
          }}>
            $3.37M
          </p>
        </div>
        <div>
          <p style={{
            fontSize: designSystem.typography.fontSize.sm,
            color: designSystem.colors.neutral[600],
            marginBottom: designSystem.spacing[1],
          }}>
            Conversion Rate
          </p>
          <p style={{
            fontSize: designSystem.typography.fontSize['2xl'],
            fontWeight: designSystem.typography.fontWeight.bold,
            color: designSystem.colors.success[600],
          }}>
            67%
          </p>
        </div>
      </div>
    </div>
  );
};

// Recent Activity Feed
interface Activity {
  id: string;
  type: 'settlement' | 'filing' | 'payment' | 'meeting';
  title: string;
  description: string;
  time: string;
  amount?: string;
}

const ActivityFeed: React.FC = () => {
  const activities: Activity[] = [
    {
      id: '1',
      type: 'settlement',
      title: 'Settlement Reached',
      description: 'Smith v. ABC Corp - $450,000',
      time: '2 hours ago',
      amount: '$450,000',
    },
    {
      id: '2',
      type: 'filing',
      title: 'Motion Filed',
      description: 'Motion for Summary Judgment - Johnson case',
      time: '4 hours ago',
    },
    {
      id: '3',
      type: 'payment',
      title: 'Payment Received',
      description: 'Invoice #1234 paid by Williams',
      time: '5 hours ago',
      amount: '$12,500',
    },
    {
      id: '4',
      type: 'meeting',
      title: 'Client Meeting',
      description: 'Initial consultation - Garcia',
      time: 'Yesterday',
    },
  ];

  const getActivityIcon = (type: Activity['type']) => {
    switch (type) {
      case 'settlement': return '🤝';
      case 'filing': return '📄';
      case 'payment': return '💰';
      case 'meeting': return '👥';
    }
  };

  return (
    <div style={{
      background: 'white',
      borderRadius: designSystem.borderRadius.lg,
      padding: designSystem.spacing[6],
      boxShadow: designSystem.shadows.md,
    }}>
      <h3 style={{
        fontSize: designSystem.typography.fontSize.xl,
        fontWeight: designSystem.typography.fontWeight.bold,
        color: designSystem.colors.neutral[900],
        marginBottom: designSystem.spacing[6],
        fontFamily: designSystem.typography.fontFamily.display,
      }}>
        Recent Activity
      </h3>

      <div style={{ display: 'flex', flexDirection: 'column', gap: designSystem.spacing[4] }}>
        {activities.map((activity) => (
          <div key={activity.id} style={{
            display: 'flex',
            gap: designSystem.spacing[4],
            padding: designSystem.spacing[3],
            borderRadius: designSystem.borderRadius.base,
            transition: designSystem.transitions.base,
            cursor: 'pointer',
            ':hover': {
              background: designSystem.colors.neutral[50],
            }
          }}>
            <div style={{
              width: '40px',
              height: '40px',
              borderRadius: designSystem.borderRadius.base,
              background: designSystem.colors.primary[50],
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: designSystem.typography.fontSize.xl,
              flexShrink: 0,
            }}>
              {getActivityIcon(activity.type)}
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                <div>
                  <p style={{
                    fontSize: designSystem.typography.fontSize.sm,
                    fontWeight: designSystem.typography.fontWeight.semibold,
                    color: designSystem.colors.neutral[900],
                    marginBottom: designSystem.spacing[1],
                  }}>
                    {activity.title}
                  </p>
                  <p style={{
                    fontSize: designSystem.typography.fontSize.sm,
                    color: designSystem.colors.neutral[600],
                  }}>
                    {activity.description}
                  </p>
                </div>
                {activity.amount && (
                  <p style={{
                    fontSize: designSystem.typography.fontSize.sm,
                    fontWeight: designSystem.typography.fontWeight.bold,
                    color: designSystem.colors.success[600],
                  }}>
                    {activity.amount}
                  </p>
                )}
              </div>
              <p style={{
                fontSize: designSystem.typography.fontSize.xs,
                color: designSystem.colors.neutral[500],
                marginTop: designSystem.spacing[1],
              }}>
                {activity.time}
              </p>
            </div>
          </div>
        ))}
      </div>

      <button style={{
        width: '100%',
        marginTop: designSystem.spacing[4],
        padding: designSystem.spacing[3],
        background: 'transparent',
        border: `2px solid ${designSystem.colors.primary[500]}`,
        borderRadius: designSystem.borderRadius.base,
        color: designSystem.colors.primary[600],
        fontWeight: designSystem.typography.fontWeight.semibold,
        fontSize: designSystem.typography.fontSize.sm,
        cursor: 'pointer',
        transition: designSystem.transitions.base,
      }}>
        View All Activity
      </button>
    </div>
  );
};

// Main Executive Dashboard
export const ExecutiveDashboard: React.FC = () => {
  return (
    <div style={{
      minHeight: '100vh',
      background: designSystem.colors.neutral[50],
      padding: designSystem.spacing[8],
    }}>
      {/* Header */}
      <div style={{ marginBottom: designSystem.spacing[8] }}>
        <h1 style={{
          fontSize: designSystem.typography.fontSize['5xl'],
          fontWeight: designSystem.typography.fontWeight.black,
          color: designSystem.colors.neutral[900],
          marginBottom: designSystem.spacing[2],
          fontFamily: designSystem.typography.fontFamily.display,
          background: `linear-gradient(135deg, ${designSystem.colors.primary[600]} 0%, ${designSystem.colors.gold[600]} 100%)`,
          WebkitBackgroundClip: 'text',
          WebkitTextFillColor: 'transparent',
        }}>
          Executive Dashboard
        </h1>
        <p style={{
          fontSize: designSystem.typography.fontSize.lg,
          color: designSystem.colors.neutral[600],
        }}>
          Welcome back! Here's what's happening with your practice today.
        </p>
      </div>

      {/* Executive Metrics */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))',
        gap: designSystem.spacing[6],
        marginBottom: designSystem.spacing[8],
      }}>
        <MetricCard
          title="Revenue (MTD)"
          value="$487K"
          change="+18.2%"
          changeType="positive"
          icon="💰"
        />
        <MetricCard
          title="Billable Hours"
          value="1,247"
          change="+12.5%"
          changeType="positive"
          icon="⏱️"
        />
        <MetricCard
          title="Collection Rate"
          value="94.3%"
          change="+2.1%"
          changeType="positive"
          icon="📊"
        />
        <MetricCard
          title="Active Matters"
          value="84"
          change="+5"
          changeType="positive"
          icon="⚖️"
        />
      </div>

      {/* Pipeline and Activity */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: '2fr 1fr',
        gap: designSystem.spacing[6],
        marginBottom: designSystem.spacing[8],
      }}>
        <PipelineView />
        <ActivityFeed />
      </div>

      {/* Quick Actions */}
      <div style={{
        background: 'white',
        borderRadius: designSystem.borderRadius.lg,
        padding: designSystem.spacing[6],
        boxShadow: designSystem.shadows.md,
      }}>
        <h3 style={{
          fontSize: designSystem.typography.fontSize.xl,
          fontWeight: designSystem.typography.fontWeight.bold,
          color: designSystem.colors.neutral[900],
          marginBottom: designSystem.spacing[6],
          fontFamily: designSystem.typography.fontFamily.display,
        }}>
          Quick Actions
        </h3>

        <div style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
          gap: designSystem.spacing[4],
        }}>
          {[
            { icon: '📝', label: 'New Matter', color: designSystem.colors.primary[500] },
            { icon: '💵', label: 'Create Invoice', color: designSystem.colors.gold[500] },
            { icon: '🔍', label: 'Legal Research', color: designSystem.colors.info[500] },
            { icon: '⚖️', label: 'Settlement Calculator', color: designSystem.colors.success[500] },
            { icon: '📧', label: 'Send Demand', color: designSystem.colors.danger[500] },
            { icon: '📊', label: 'View Reports', color: designSystem.colors.neutral[700] },
          ].map((action) => (
            <button key={action.label} style={{
              padding: designSystem.spacing[4],
              background: `linear-gradient(135deg, ${action.color} 0%, ${action.color}dd 100%)`,
              border: 'none',
              borderRadius: designSystem.borderRadius.lg,
              color: 'white',
              fontSize: designSystem.typography.fontSize.sm,
              fontWeight: designSystem.typography.fontWeight.semibold,
              cursor: 'pointer',
              transition: designSystem.transitions.base,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              gap: designSystem.spacing[2],
              boxShadow: `0 4px 6px -1px ${action.color}40`,
            }}>
              <span style={{ fontSize: designSystem.typography.fontSize['3xl'] }}>
                {action.icon}
              </span>
              {action.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};

export default ExecutiveDashboard;
