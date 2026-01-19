import React from 'react';
import { View, Text, StyleSheet, StatusBar, ViewStyle } from 'react-native';
import { BlurView } from '@react-native-community/blur';

interface GlassHeaderProps {
  title: string;
  style?: ViewStyle;
  blurAmount?: number;
  rightComponent?: React.ReactNode;
}

export function GlassHeader({
  title,
  style,
  blurAmount = 10,
  rightComponent,
}: GlassHeaderProps) {
  return (
    <>
      <StatusBar barStyle="light-content" translucent />
      <BlurView
        style={[styles.header, style]}
        blurType="light"
        blurAmount={blurAmount}
        reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
      >
        <View style={styles.content}>
          <Text style={styles.title}>{title}</Text>
          {rightComponent && <View>{rightComponent}</View>}
        </View>
      </BlurView>
    </>
  );
}

const styles = StyleSheet.create({
  header: {
    paddingTop: StatusBar.currentHeight || 0,
    borderBottomWidth: 1,
    borderBottomColor: 'rgba(255, 255, 255, 0.1)',
  },
  content: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingVertical: 12,
  },
  title: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#FFFFFF',
  },
});
