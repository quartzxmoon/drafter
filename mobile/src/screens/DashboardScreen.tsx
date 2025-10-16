// Dashboard Screen - Main overview for clients

import React from 'react';
import {
  View,
  ScrollView,
  StyleSheet,
  RefreshControl,
  TouchableOpacity,
} from 'react-native';
import {
  Card,
  Title,
  Paragraph,
  Button,
  List,
  Badge,
  Chip,
  Surface,
} from 'react-native-paper';
import { useQuery } from '@tanstack/react-query';
import Icon from 'react-native-vector-icons/MaterialCommunityIcons';
import { useNavigation } from '@react-navigation/native';
import { apiClient } from '../services/api';

interface DashboardData {
  client_id: string;
  matters: Matter[];
  recent_documents: Document[];
  unread_messages: number;
  pending_signatures: number;
  upcoming_deadlines: Deadline[];
  recent_activity: Activity[];
}

interface Matter {
  id: string;
  title: string;
  matter_number: string;
  status: string;
  docket_number?: string;
  court_name?: string;
  next_deadline?: string;
  document_count: number;
  unread_messages: number;
}

interface Document {
  id: string;
  title: string;
  shared_at: string;
  requires_signature: boolean;
  file_size: number;
}

interface Deadline {
  id: string;
  matter_id: string;
  title: string;
  deadline_date: string;
  days_remaining: number;
  priority: string;
}

interface Activity {
  id: string;
  activity_type: string;
  description: string;
  created_at: string;
}

export default function DashboardScreen() {
  const navigation = useNavigation();

  const { data, isLoading, refetch } = useQuery<DashboardData>({
    queryKey: ['dashboard'],
    queryFn: () => apiClient.get('/portal/dashboard'),
  });

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  };

  const getPriorityColor = (priority: string) => {
    switch (priority.toLowerCase()) {
      case 'critical':
        return '#ef4444';
      case 'high':
        return '#f97316';
      case 'medium':
        return '#eab308';
      default:
        return '#64748b';
    }
  };

  return (
    <ScrollView
      style={styles.container}
      refreshControl={
        <RefreshControl refreshing={isLoading} onRefresh={refetch} />
      }
    >
      {/* Header */}
      <View style={styles.header}>
        <Title style={styles.headerTitle}>Dashboard</Title>
      </View>

      {/* Quick Stats */}
      <View style={styles.statsContainer}>
        <Surface style={styles.statCard}>
          <Icon name="briefcase" size={24} color="#2563eb" />
          <Paragraph style={styles.statNumber}>{data?.matters.length || 0}</Paragraph>
          <Paragraph style={styles.statLabel}>Active Cases</Paragraph>
        </Surface>

        <Surface style={styles.statCard}>
          <Icon name="file-document" size={24} color="#16a34a" />
          <Paragraph style={styles.statNumber}>{data?.recent_documents.length || 0}</Paragraph>
          <Paragraph style={styles.statLabel}>Documents</Paragraph>
        </Surface>

        <Surface style={styles.statCard}>
          <Icon name="message" size={24} color="#dc2626" />
          <Paragraph style={styles.statNumber}>{data?.unread_messages || 0}</Paragraph>
          <Paragraph style={styles.statLabel}>Unread</Paragraph>
        </Surface>

        <Surface style={styles.statCard}>
          <Icon name="clock-alert" size={24} color="#ea580c" />
          <Paragraph style={styles.statNumber}>{data?.upcoming_deadlines.length || 0}</Paragraph>
          <Paragraph style={styles.statLabel}>Deadlines</Paragraph>
        </Surface>
      </View>

      {/* Pending Signatures Alert */}
      {data && data.pending_signatures > 0 && (
        <Card style={styles.alertCard}>
          <Card.Content style={styles.alertContent}>
            <Icon name="alert-circle" size={32} color="#dc2626" />
            <View style={styles.alertText}>
              <Title style={styles.alertTitle}>
                {data.pending_signatures} Document{data.pending_signatures > 1 ? 's' : ''} Awaiting Signature
              </Title>
              <Paragraph>Please review and sign pending documents</Paragraph>
            </View>
            <Button mode="contained" onPress={() => navigation.navigate('Documents' as never)}>
              Review
            </Button>
          </Card.Content>
        </Card>
      )}

      {/* Upcoming Deadlines */}
      {data && data.upcoming_deadlines.length > 0 && (
        <Card style={styles.section}>
          <Card.Title
            title="Upcoming Deadlines"
            right={(props) => (
              <Button {...props} onPress={() => navigation.navigate('Deadlines' as never)}>
                View All
              </Button>
            )}
          />
          <Card.Content>
            {data.upcoming_deadlines.slice(0, 3).map((deadline) => (
              <TouchableOpacity
                key={deadline.id}
                style={styles.deadlineItem}
                onPress={() =>
                  navigation.navigate('MatterDetail' as never, {
                    matterId: deadline.matter_id,
                  } as never)
                }
              >
                <View style={styles.deadlineInfo}>
                  <Paragraph style={styles.deadlineTitle}>{deadline.title}</Paragraph>
                  <Paragraph style={styles.deadlineDate}>
                    {formatDate(deadline.deadline_date)} •{' '}
                    {deadline.days_remaining} day{deadline.days_remaining !== 1 ? 's' : ''} remaining
                  </Paragraph>
                </View>
                <Chip
                  style={[
                    styles.priorityChip,
                    { backgroundColor: getPriorityColor(deadline.priority) },
                  ]}
                  textStyle={styles.priorityText}
                >
                  {deadline.priority}
                </Chip>
              </TouchableOpacity>
            ))}
          </Card.Content>
        </Card>
      )}

      {/* Active Matters */}
      {data && data.matters.length > 0 && (
        <Card style={styles.section}>
          <Card.Title
            title="Your Cases"
            right={(props) => (
              <Button {...props} onPress={() => navigation.navigate('Matters' as never)}>
                View All
              </Button>
            )}
          />
          <Card.Content>
            {data.matters.slice(0, 3).map((matter) => (
              <TouchableOpacity
                key={matter.id}
                style={styles.matterItem}
                onPress={() =>
                  navigation.navigate('MatterDetail' as never, {
                    matterId: matter.id,
                  } as never)
                }
              >
                <View style={styles.matterHeader}>
                  <View style={styles.matterInfo}>
                    <Title style={styles.matterTitle}>{matter.title}</Title>
                    <Paragraph style={styles.matterMeta}>
                      {matter.matter_number}
                      {matter.docket_number && ` • ${matter.docket_number}`}
                    </Paragraph>
                    {matter.court_name && (
                      <Paragraph style={styles.courtName}>{matter.court_name}</Paragraph>
                    )}
                  </View>
                  <Chip style={styles.statusChip}>{matter.status}</Chip>
                </View>
                <View style={styles.matterStats}>
                  <View style={styles.matterStat}>
                    <Icon name="file-document" size={16} color="#64748b" />
                    <Paragraph style={styles.matterStatText}>
                      {matter.document_count} docs
                    </Paragraph>
                  </View>
                  {matter.unread_messages > 0 && (
                    <View style={styles.matterStat}>
                      <Icon name="message" size={16} color="#dc2626" />
                      <Paragraph style={styles.matterStatText}>
                        {matter.unread_messages} unread
                      </Paragraph>
                    </View>
                  )}
                </View>
              </TouchableOpacity>
            ))}
          </Card.Content>
        </Card>
      )}

      {/* Recent Documents */}
      {data && data.recent_documents.length > 0 && (
        <Card style={styles.section}>
          <Card.Title
            title="Recent Documents"
            right={(props) => (
              <Button {...props} onPress={() => navigation.navigate('Documents' as never)}>
                View All
              </Button>
            )}
          />
          <Card.Content>
            {data.recent_documents.slice(0, 3).map((doc) => (
              <List.Item
                key={doc.id}
                title={doc.title}
                description={`Shared ${formatDate(doc.shared_at)}`}
                left={(props) => (
                  <List.Icon
                    {...props}
                    icon={doc.requires_signature ? 'file-sign' : 'file-pdf-box'}
                    color={doc.requires_signature ? '#dc2626' : '#2563eb'}
                  />
                )}
                right={(props) =>
                  doc.requires_signature ? (
                    <Badge {...props} style={styles.signatureBadge}>
                      Sign
                    </Badge>
                  ) : null
                }
                onPress={() =>
                  navigation.navigate('DocumentViewer' as never, {
                    documentId: doc.id,
                  } as never)
                }
                style={styles.documentItem}
              />
            ))}
          </Card.Content>
        </Card>
      )}

      {/* Recent Activity */}
      {data && data.recent_activity.length > 0 && (
        <Card style={[styles.section, styles.lastSection]}>
          <Card.Title title="Recent Activity" />
          <Card.Content>
            {data.recent_activity.slice(0, 5).map((activity) => (
              <List.Item
                key={activity.id}
                title={activity.description}
                description={formatDate(activity.created_at)}
                left={(props) => (
                  <List.Icon {...props} icon="history" color="#64748b" />
                )}
                style={styles.activityItem}
              />
            ))}
          </Card.Content>
        </Card>
      )}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f8fafc',
  },
  header: {
    padding: 20,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e2e8f0',
  },
  headerTitle: {
    fontSize: 28,
    fontWeight: 'bold',
  },
  statsContainer: {
    flexDirection: 'row',
    padding: 16,
    gap: 12,
  },
  statCard: {
    flex: 1,
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
    elevation: 2,
  },
  statNumber: {
    fontSize: 24,
    fontWeight: 'bold',
    marginTop: 8,
  },
  statLabel: {
    fontSize: 12,
    color: '#64748b',
    marginTop: 4,
  },
  alertCard: {
    margin: 16,
    marginTop: 0,
    backgroundColor: '#fef2f2',
    borderColor: '#dc2626',
    borderWidth: 1,
  },
  alertContent: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 16,
  },
  alertText: {
    flex: 1,
  },
  alertTitle: {
    fontSize: 16,
    color: '#dc2626',
  },
  section: {
    margin: 16,
    marginTop: 0,
  },
  lastSection: {
    marginBottom: 32,
  },
  deadlineItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 12,
    borderRadius: 8,
    backgroundColor: '#f8fafc',
    marginBottom: 8,
  },
  deadlineInfo: {
    flex: 1,
  },
  deadlineTitle: {
    fontWeight: '600',
    marginBottom: 4,
  },
  deadlineDate: {
    fontSize: 12,
    color: '#64748b',
  },
  priorityChip: {
    height: 24,
  },
  priorityText: {
    color: '#fff',
    fontSize: 11,
    fontWeight: '600',
  },
  matterItem: {
    padding: 16,
    borderRadius: 8,
    backgroundColor: '#f8fafc',
    marginBottom: 12,
  },
  matterHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 12,
  },
  matterInfo: {
    flex: 1,
  },
  matterTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 4,
  },
  matterMeta: {
    fontSize: 12,
    color: '#64748b',
  },
  courtName: {
    fontSize: 12,
    color: '#2563eb',
    marginTop: 4,
  },
  statusChip: {
    height: 28,
    marginLeft: 12,
  },
  matterStats: {
    flexDirection: 'row',
    gap: 16,
  },
  matterStat: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 4,
  },
  matterStatText: {
    fontSize: 12,
    color: '#64748b',
  },
  documentItem: {
    paddingVertical: 8,
  },
  signatureBadge: {
    backgroundColor: '#dc2626',
    color: '#fff',
  },
  activityItem: {
    paddingVertical: 4,
  },
});
