import React from 'react';
import { View, StyleSheet, Dimensions, ViewStyle } from 'react-native';
import { BlurView } from '@react-native-community/blur';

interface GlassScreenProps {
  children: React.ReactNode;
  style?: ViewStyle;
  blurAmount?: number;
  backgroundColor?: string;
}

const { width, height } = Dimensions.get('window');

export function GlassScreen({
  children,
  style,
  blurAmount = 15,
  backgroundColor = 'rgba(255, 255, 255, 0.1)',
}: GlassScreenProps) {
  return (
    <View style={styles.container}>
      <BlurView
        style={[styles.overlay, { backgroundColor }, style]}
        blurType="light"
        blurAmount={blurAmount}
        reducedTransparencyFallbackColor={backgroundColor}
      >
        <View style={styles.content}>{children}</View>
      </BlurView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  overlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    width,
    height,
  },
  content: {
    flex: 1,
    padding: 20,
  },
});
