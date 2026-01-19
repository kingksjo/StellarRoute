# Mobile Glassmorphism Patterns

Comprehensive patterns for implementing glassmorphic effects in React Native applications.

## Table of Contents

- [BlurView Implementation](#blurview-implementation)
- [NativeWind Integration](#nativewind-integration)
- [shadcn-native Integration](#shadcn-native-integration)
- [Component Patterns](#component-patterns)
- [Performance Optimization](#performance-optimization)
- [Platform-Specific Considerations](#platform-specific-considerations)

## BlurView Implementation

### Installation

```bash
npm install @react-native-community/blur
# or
yarn add @react-native-community/blur

# iOS only - requires pod install
cd ios && pod install
```

### Basic Usage

```tsx
import { BlurView } from '@react-native-community/blur';
import { View, Text, StyleSheet } from 'react-native';

function GlassCard() {
  return (
    <BlurView
      style={styles.container}
      blurType="light"
      blurAmount={10}
      reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
    >
      <View style={styles.content}>
        <Text style={styles.title}>Glass Card</Text>
        <Text style={styles.text}>Content with glass effect</Text>
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
  text: {
    fontSize: 14,
    color: 'rgba(255, 255, 255, 0.8)',
  },
});
```

### Blur Types

**iOS:**
- `light` - Light blur (recommended for dark backgrounds)
- `dark` - Dark blur (recommended for light backgrounds)
- `xlight` - Extra light blur
- `prominent` - Prominent blur

**Android:**
- `dark` - Dark blur
- `light` - Light blur
- `xlight` - Extra light blur

### Blur Amount

```tsx
// Subtle blur
<BlurView blurAmount={5} blurType="light" />

// Medium blur (recommended)
<BlurView blurAmount={10} blurType="light" />

// Strong blur
<BlurView blurAmount={20} blurType="light" />
```

## NativeWind Integration

### Setup

```bash
npm install nativewind
npm install --save-dev tailwindcss
npx tailwindcss init
```

### Configuration

```js
// tailwind.config.js
module.exports = {
  content: [
    "./App.{js,jsx,ts,tsx}",
    "./src/**/*.{js,jsx,ts,tsx}"
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
```

### Glass Components with NativeWind

```tsx
import { BlurView } from '@react-native-community/blur';
import { View, Text } from 'react-native';

function GlassCard() {
  return (
    <BlurView
      className="rounded-2xl overflow-hidden border border-white/20"
      blurType="light"
      blurAmount={10}
    >
      <View className="p-6">
        <Text className="text-white text-lg font-semibold mb-2">
          Glass Card
        </Text>
        <Text className="text-white/80">
          Content with glass effect
        </Text>
      </View>
    </BlurView>
  );
}
```

**Note**: NativeWind doesn't support `backdrop-filter` directly. Use BlurView for blur effects and NativeWind for styling.

## shadcn-native Integration

### Installation

```bash
# Install NativeWind first (required)
npm install nativewind
npm install --save-dev tailwindcss

# Install shadcn-native
npm install shadcn-native

# Initialize
npx shadcn-native@latest init
```

### Glass Card Variant

```tsx
import { Card, CardHeader, CardTitle, CardContent } from 'shadcn-native';
import { BlurView } from '@react-native-community/blur';

function GlassCard() {
  return (
    <BlurView
      style={{ borderRadius: 12, overflow: 'hidden' }}
      blurType="light"
      blurAmount={10}
    >
      <Card className="bg-white/10 border-white/20">
        <CardHeader>
          <CardTitle className="text-white">Glass Card</CardTitle>
        </CardHeader>
        <CardContent className="text-white/80">
          Content with glass effect
        </CardContent>
      </Card>
    </BlurView>
  );
}
```

## Component Patterns

### Glass Card

```tsx
import { BlurView } from '@react-native-community/blur';
import { View, Text, StyleSheet } from 'react-native';

interface GlassCardProps {
  title: string;
  content: string;
}

export function GlassCard({ title, content }: GlassCardProps) {
  return (
    <BlurView
      style={styles.container}
      blurType="light"
      blurAmount={10}
      reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
    >
      <View style={styles.content}>
        <Text style={styles.title}>{title}</Text>
        <Text style={styles.text}>{content}</Text>
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
  text: {
    fontSize: 14,
    color: 'rgba(255, 255, 255, 0.8)',
  },
});
```

### Glass Screen Overlay

```tsx
import { BlurView } from '@react-native-community/blur';
import { View, StyleSheet, Dimensions } from 'react-native';

const { width, height } = Dimensions.get('window');

export function GlassOverlay({ children }: { children: React.ReactNode }) {
  return (
    <View style={styles.container}>
      {/* Background content */}
      <View style={styles.background}>
        {/* Your background content here */}
      </View>
      
      {/* Glass overlay */}
      <BlurView
        style={styles.overlay}
        blurType="light"
        blurAmount={15}
        reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
      >
        <View style={styles.content}>
          {children}
        </View>
      </BlurView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  background: {
    position: 'absolute',
    width,
    height,
  },
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(255, 255, 255, 0.1)',
  },
  content: {
    flex: 1,
    padding: 20,
  },
});
```

### Glass Modal

```tsx
import { BlurView } from '@react-native-community/blur';
import { Modal, View, Text, TouchableOpacity, StyleSheet } from 'react-native';

interface GlassModalProps {
  visible: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

export function GlassModal({ visible, onClose, title, children }: GlassModalProps) {
  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      onRequestClose={onClose}
    >
      <View style={styles.backdrop}>
        <BlurView
          style={styles.blur}
          blurType="dark"
          blurAmount={10}
        />
        <View style={styles.modalContainer}>
          <BlurView
            style={styles.modal}
            blurType="light"
            blurAmount={15}
            reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
          >
            <View style={styles.header}>
              <Text style={styles.title}>{title}</Text>
              <TouchableOpacity onPress={onClose}>
                <Text style={styles.closeButton}>✕</Text>
              </TouchableOpacity>
            </View>
            <View style={styles.content}>
              {children}
            </View>
          </BlurView>
        </View>
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  backdrop: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
  },
  blur: {
    position: 'absolute',
    top: 0,
    left: 0,
    bottom: 0,
    right: 0,
  },
  modalContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 20,
  },
  modal: {
    width: '100%',
    maxWidth: 400,
    borderRadius: 20,
    overflow: 'hidden',
    borderWidth: 1,
    borderColor: 'rgba(255, 255, 255, 0.2)',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 8 },
    shadowOpacity: 0.3,
    shadowRadius: 16,
    elevation: 16,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 20,
    borderBottomWidth: 1,
    borderBottomColor: 'rgba(255, 255, 255, 0.1)',
  },
  title: {
    fontSize: 20,
    fontWeight: 'bold',
    color: '#FFFFFF',
  },
  closeButton: {
    fontSize: 24,
    color: 'rgba(255, 255, 255, 0.8)',
  },
  content: {
    padding: 20,
  },
});
```

### Glass Navigation Header

```tsx
import { BlurView } from '@react-native-community/blur';
import { View, Text, StyleSheet, StatusBar } from 'react-native';

export function GlassHeader({ title }: { title: string }) {
  return (
    <>
      <StatusBar barStyle="light-content" translucent />
      <BlurView
        style={styles.header}
        blurType="light"
        blurAmount={10}
        reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
      >
        <View style={styles.content}>
          <Text style={styles.title}>{title}</Text>
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
    paddingHorizontal: 16,
    paddingVertical: 12,
  },
  title: {
    fontSize: 18,
    fontWeight: 'bold',
    color: '#FFFFFF',
  },
});
```

### Glass Button

```tsx
import { BlurView } from '@react-native-community/blur';
import { TouchableOpacity, Text, StyleSheet, ActivityIndicator } from 'react-native';

interface GlassButtonProps {
  title: string;
  onPress: () => void;
  loading?: boolean;
  variant?: 'primary' | 'secondary';
}

export function GlassButton({ 
  title, 
  onPress, 
  loading = false,
  variant = 'primary' 
}: GlassButtonProps) {
  return (
    <TouchableOpacity
      onPress={onPress}
      disabled={loading}
      activeOpacity={0.8}
    >
      <BlurView
        style={[
          styles.button,
          variant === 'secondary' && styles.buttonSecondary
        ]}
        blurType="light"
        blurAmount={8}
        reducedTransparencyFallbackColor="rgba(255, 255, 255, 0.1)"
      >
        {loading ? (
          <ActivityIndicator color="#FFFFFF" />
        ) : (
          <Text style={styles.buttonText}>{title}</Text>
        )}
      </BlurView>
    </TouchableOpacity>
  );
}

const styles = StyleSheet.create({
  button: {
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 12,
    borderWidth: 1,
    borderColor: 'rgba(255, 255, 255, 0.2)',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 44,
  },
  buttonSecondary: {
    backgroundColor: 'transparent',
    borderWidth: 2,
    borderColor: 'rgba(255, 255, 255, 0.3)',
  },
  buttonText: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
});
```

## Performance Optimization

### Conditional Rendering

```tsx
import { Platform } from 'react-native';
import { BlurView } from '@react-native-community/blur';

function ConditionalGlass({ children }: { children: React.ReactNode }) {
  // Use BlurView on iOS, fallback on Android
  if (Platform.OS === 'ios') {
    return (
      <BlurView blurType="light" blurAmount={10}>
        {children}
      </BlurView>
    );
  }
  
  // Android fallback
  return (
    <View style={{ backgroundColor: 'rgba(255, 255, 255, 0.1)' }}>
      {children}
    </View>
  );
}
```

### Reduce Blur on Low-End Devices

```tsx
import { useMemo } from 'react';
import { Platform, PixelRatio } from 'react-native';

function useBlurAmount() {
  return useMemo(() => {
    const pixelRatio = PixelRatio.get();
    
    // Reduce blur on low-end devices
    if (pixelRatio < 2) {
      return 5; // Subtle blur
    } else if (pixelRatio < 3) {
      return 10; // Medium blur
    } else {
      return 15; // Strong blur
    }
  }, []);
}

function GlassCard() {
  const blurAmount = useBlurAmount();
  
  return (
    <BlurView blurType="light" blurAmount={blurAmount}>
      {/* Content */}
    </BlurView>
  );
}
```

### Memoize Styles

```tsx
import { useMemo } from 'react';
import { StyleSheet } from 'react-native';

function GlassCard({ intensity }: { intensity: 'subtle' | 'medium' | 'strong' }) {
  const styles = useMemo(() => {
    const blurAmounts = {
      subtle: 5,
      medium: 10,
      strong: 15,
    };
    
    return StyleSheet.create({
      container: {
        borderRadius: 16,
        borderWidth: 1,
        borderColor: `rgba(255, 255, 255, ${intensity === 'strong' ? 0.3 : 0.2})`,
      },
    });
  }, [intensity]);
  
  return (
    <BlurView
      style={styles.container}
      blurType="light"
      blurAmount={blurAmounts[intensity]}
    >
      {/* Content */}
    </BlurView>
  );
}
```

## Platform-Specific Considerations

### iOS

- BlurView works natively with excellent performance
- Use `blurType` based on background (light/dark)
- `reducedTransparencyFallbackColor` for accessibility

### Android

- BlurView uses RenderScript (may be deprecated)
- Consider alternative libraries for better Android support
- Test on various Android versions
- May need fallback to semi-transparent backgrounds

### Alternative Libraries

**react-native-blur:**
```bash
npm install react-native-blur
```

**react-native-fast-blur:**
```bash
npm install react-native-fast-blur
```

## Best Practices

1. **Test on Real Devices**: Blur effects can vary significantly between devices
2. **Provide Fallbacks**: Always have a fallback for unsupported platforms
3. **Limit Blur Intensity**: Higher blur amounts impact performance
4. **Use Selectively**: Don't apply glass effects to every component
5. **Consider Accessibility**: Ensure sufficient contrast for readability
6. **Optimize Re-renders**: Memoize components and styles when possible
