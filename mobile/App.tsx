// PA eDocket Mobile - Main App Component
// React Native mobile companion app for iOS and Android

import React, { useEffect } from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createStackNavigator } from '@react-navigation/stack';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { Provider as PaperProvider } from 'react-native-paper';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import Icon from 'react-native-vector-icons/MaterialCommunityIcons';
import * as SecureStore from 'expo-secure-store';
import * as LocalAuthentication from 'expo-local-authentication';

// Screens
import LoginScreen from './src/screens/LoginScreen';
import DashboardScreen from './src/screens/DashboardScreen';
import MattersScreen from './src/screens/MattersScreen';
import MatterDetailScreen from './src/screens/MatterDetailScreen';
import DocumentsScreen from './src/screens/DocumentsScreen';
import DocumentViewerScreen from './src/screens/DocumentViewerScreen';
import MessagesScreen from './src/screens/MessagesScreen';
import MessageDetailScreen from './src/screens/MessageDetailScreen';
import DeadlinesScreen from './src/screens/DeadlinesScreen';
import NotificationsScreen from './src/screens/NotificationsScreen';
import SettingsScreen from './src/screens/SettingsScreen';
import SignatureScreen from './src/screens/SignatureScreen';
import ScanDocumentScreen from './src/screens/ScanDocumentScreen';

// Types
export type RootStackParamList = {
  Login: undefined;
  Main: undefined;
  MatterDetail: { matterId: string };
  DocumentViewer: { documentId: string };
  MessageDetail: { messageId: string };
  Signature: { documentId: string };
  ScanDocument: undefined;
};

export type MainTabParamList = {
  Dashboard: undefined;
  Matters: undefined;
  Documents: undefined;
  Messages: undefined;
  More: undefined;
};

const Stack = createStackNavigator<RootStackParamList>();
const Tab = createBottomTabNavigator<MainTabParamList>();
const queryClient = new QueryClient();

function MainTabs() {
  return (
    <Tab.Navigator
      screenOptions={({ route }) => ({
        tabBarIcon: ({ focused, color, size }) => {
          let iconName: string;

          switch (route.name) {
            case 'Dashboard':
              iconName = focused ? 'view-dashboard' : 'view-dashboard-outline';
              break;
            case 'Matters':
              iconName = focused ? 'briefcase' : 'briefcase-outline';
              break;
            case 'Documents':
              iconName = focused ? 'file-document' : 'file-document-outline';
              break;
            case 'Messages':
              iconName = focused ? 'message' : 'message-outline';
              break;
            case 'More':
              iconName = focused ? 'menu' : 'menu';
              break;
            default:
              iconName = 'circle';
          }

          return <Icon name={iconName} size={size} color={color} />;
        },
        tabBarActiveTintColor: '#2563eb',
        tabBarInactiveTintColor: 'gray',
        headerShown: false,
      })}
    >
      <Tab.Screen
        name="Dashboard"
        component={DashboardScreen}
        options={{ title: 'Dashboard' }}
      />
      <Tab.Screen
        name="Matters"
        component={MattersScreen}
        options={{ title: 'My Cases' }}
      />
      <Tab.Screen
        name="Documents"
        component={DocumentsScreen}
        options={{ title: 'Documents' }}
      />
      <Tab.Screen
        name="Messages"
        component={MessagesScreen}
        options={{ title: 'Messages' }}
      />
      <Tab.Screen
        name="More"
        component={MoreNavigator}
        options={{ title: 'More' }}
      />
    </Tab.Navigator>
  );
}

const MoreStack = createStackNavigator();

function MoreNavigator() {
  return (
    <MoreStack.Navigator>
      <MoreStack.Screen name="MoreHome" component={SettingsScreen} options={{ title: 'More' }} />
      <MoreStack.Screen name="Deadlines" component={DeadlinesScreen} options={{ title: 'Deadlines' }} />
      <MoreStack.Screen name="Notifications" component={NotificationsScreen} options={{ title: 'Notifications' }} />
    </MoreStack.Navigator>
  );
}

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = React.useState(false);
  const [isLoading, setIsLoading] = React.useState(true);

  useEffect(() => {
    checkAuthentication();
  }, []);

  const checkAuthentication = async () => {
    try {
      // Check for stored session token
      const token = await SecureStore.getItemAsync('session_token');

      if (token) {
        // Optionally verify with biometric authentication
        const hasHardware = await LocalAuthentication.hasHardwareAsync();
        const isEnrolled = await LocalAuthentication.isEnrolledAsync();

        if (hasHardware && isEnrolled) {
          const result = await LocalAuthentication.authenticateAsync({
            promptMessage: 'Authenticate to access PA eDocket',
            fallbackLabel: 'Use passcode',
          });

          if (result.success) {
            setIsAuthenticated(true);
          }
        } else {
          setIsAuthenticated(true);
        }
      }
    } catch (error) {
      console.error('Authentication check failed:', error);
    } finally {
      setIsLoading(false);
    }
  };

  if (isLoading) {
    return null; // Or show a splash screen
  }

  return (
    <QueryClientProvider client={queryClient}>
      <PaperProvider>
        <NavigationContainer>
          <Stack.Navigator
            screenOptions={{
              headerShown: false,
            }}
          >
            {isAuthenticated ? (
              <>
                <Stack.Screen name="Main" component={MainTabs} />
                <Stack.Screen
                  name="MatterDetail"
                  component={MatterDetailScreen}
                  options={{ headerShown: true, title: 'Case Details' }}
                />
                <Stack.Screen
                  name="DocumentViewer"
                  component={DocumentViewerScreen}
                  options={{ headerShown: true, title: 'Document' }}
                />
                <Stack.Screen
                  name="MessageDetail"
                  component={MessageDetailScreen}
                  options={{ headerShown: true, title: 'Message' }}
                />
                <Stack.Screen
                  name="Signature"
                  component={SignatureScreen}
                  options={{ headerShown: true, title: 'Sign Document' }}
                />
                <Stack.Screen
                  name="ScanDocument"
                  component={ScanDocumentScreen}
                  options={{ headerShown: true, title: 'Scan Document' }}
                />
              </>
            ) : (
              <Stack.Screen name="Login" component={LoginScreen} />
            )}
          </Stack.Navigator>
        </NavigationContainer>
      </PaperProvider>
    </QueryClientProvider>
  );
}
