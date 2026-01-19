import React from 'react';
import { View, Text, StyleSheet, ViewStyle } from 'react-native';
import { BlurView } from '@react-native-community/blur';

interface GlassCardProps {
  title?: string;
  children: React.ReactNode;
  style?: ViewStyle;
  intensity?: 'subtle' | 'medium' | 'strong';
}

const blurAmounts = {
  subtle: 5,
  medium: 10,
  strong: 15,
};

export function GlassCard({
  title,
  children,
  style,
  intensity = 'medium',
}: GlassCardProps) {
  return (
    <BlurView
      style={[styles.container, style]}
      blurType="light"
      blurAmount={blurAmounts[intensity]}
      reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
    >
      <View style={styles.content}>
        {title && <Text style={styles.title}>{title}</Text>}
        {children}
      </View>
    </BlurView>
  );
}

const styles = StyleSheet.create({
  container: {
    borderRadius: 16,
    overflow: 'hidden',
    borderWidth: 1,
    borderColor: 'rgba(255, 255, 255, 0.2)',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.3,
    shadowRadius: 8,
    elevation: 8,
  },
  content: {
    padding: 16,
  },
  title: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#FFFFFF',
    marginBottom: 8,
  },
});
