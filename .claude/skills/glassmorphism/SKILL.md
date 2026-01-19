---
name: glassmorphism
description: Transform UI components in web and mobile applications into responsive glassmorphic effects using Tailwind CSS, Shadcn UI, and React Native patterns. Use when creating or modifying components that need frosted glass aesthetics with backdrop blur, semi-transparent backgrounds, subtle borders, and depth effects. Supports web (React/Next.js with Tailwind CSS and Shadcn UI) and mobile (React Native) implementations with performance-optimized patterns.
license: Complete terms in LICENSE.txt
---

# Glassmorphism UI Effects

Transform UI components into modern glassmorphic designs with responsive, performance-optimized glass effects for web and mobile applications.

## Overview

Glassmorphism creates a frosted glass aesthetic using backdrop blur, semi-transparent backgrounds, and subtle borders. This skill provides patterns, templates, and best practices for implementing glassmorphic effects in web (Tailwind CSS + Shadcn UI) and mobile (React Native) applications.

**When to use glassmorphism:**
- Cards, modals, and overlays that need depth
- Navigation bars and headers
- Dashboard widgets and panels
- Mobile app screens with layered content
- Modern, premium UI aesthetics

## Quick Start

### Web (Tailwind CSS + Shadcn UI)

Basic glassmorphic card:

```tsx
<div className="bg-white/10 backdrop-blur-lg border border-white/20 rounded-lg p-6 shadow-lg">
  <h2 className="text-white font-semibold mb-2">Glass Card</h2>
  <p className="text-white/80">Content with glass effect</p>
</div>
```

### Mobile (React Native)

Using `@react-native-community/blur`:

```tsx
import { BlurView } from '@react-native-community/blur';

<BlurView
  style={styles.container}
  blurType="light"
  blurAmount={10}
>
  <View style={styles.content}>
    <Text>Glass Content</Text>
  </View>
</BlurView>
```

## Glassmorphism Principles

### Core Characteristics

1. **Backdrop Blur**: Creates the frosted glass effect
2. **Semi-transparent Background**: 10-20% opacity for glass appearance
3. **Subtle Borders**: 10-30% opacity borders for edge definition
4. **Soft Shadows**: Adds depth and separation
5. **Vibrant Backgrounds**: Colorful/gradient backgrounds enhance the effect

### Intensity Levels

- **Subtle**: `backdrop-blur-sm`, `bg-opacity-5`, minimal border
- **Medium**: `backdrop-blur-md` to `backdrop-blur-lg`, `bg-opacity-10`, visible border
- **Strong**: `backdrop-blur-xl` to `backdrop-blur-2xl`, `bg-opacity-15`, prominent border

## Web Implementation

### Tailwind CSS Patterns

See [web-patterns.md](references/web-patterns.md) for comprehensive Tailwind CSS patterns including:
- Component variants (cards, buttons, modals, navbars)
- Responsive adjustments
- Dark mode support
- Shadcn UI integration
- Performance optimization

### Shadcn UI Integration

Customize Shadcn components with glassmorphic variants:

```tsx
// Glassmorphic Card variant
<Card className="bg-white/10 backdrop-blur-lg border-white/20">
  <CardHeader>
    <CardTitle className="text-white">Glass Card</CardTitle>
  </CardHeader>
</Card>
```

See [web-patterns.md](references/web-patterns.md) for Shadcn component customization patterns.

### Component Templates

Pre-built templates in `assets/templates/web/`:
- `GlassCard.tsx` - Reusable glass card component
- `GlassButton.tsx` - Glassmorphic button variants
- `GlassModal.tsx` - Modal with glass effect
- `GlassNavbar.tsx` - Navigation bar template
- `GlassContainer.tsx` - Container with gradient background

## Mobile Implementation

### React Native Patterns

See [mobile-patterns.md](references/mobile-patterns.md) for React Native implementation including:
- BlurView usage patterns
- NativeWind (Tailwind for React Native) integration
- shadcn-native component customization
- Performance considerations
- Platform-specific optimizations

### Blur Libraries

**Recommended**: `@react-native-community/blur`
- Native performance
- iOS and Android support
- Multiple blur types

**Alternative**: `react-native-blur`
- Similar API
- Check compatibility with your React Native version

### Component Templates

Pre-built templates in `assets/templates/mobile/`:
- `GlassCard.tsx` - React Native glass card
- `GlassScreen.tsx` - Full screen with glass overlay
- `GlassModal.tsx` - Modal component
- `GlassHeader.tsx` - Navigation header

## Responsive Design

### Breakpoint Adjustments

Adjust glass intensity and spacing for different screen sizes:

```tsx
<div className="
  bg-white/10 backdrop-blur-md
  md:backdrop-blur-lg
  lg:backdrop-blur-xl
  p-4 md:p-6 lg:p-8
">
```

### Mobile Considerations

- Reduce blur intensity on low-end devices
- Use conditional rendering for blur support
- Provide fallbacks for unsupported browsers
- Test performance on various devices

See [performance.md](references/performance.md) for optimization strategies.

## Best Practices

### 1. Use Sparingly

Apply glassmorphism to key elements (cards, modals, navbars) rather than every component. Overuse can overwhelm the UI.

### 2. Maintain Readability

Ensure sufficient contrast between text and background:
- Use darker text on light glass
- Use lighter text on dark glass
- Test with accessibility tools

### 3. Optimize Performance

- Use GPU-accelerated properties (`transform`, `opacity`)
- Limit blur intensity on mobile
- Test on low-end devices
- Consider `will-change` for animated glass elements

### 4. Provide Fallbacks

For browsers without `backdrop-filter` support:
- Use solid backgrounds with opacity
- Show warning or alternative design
- Use feature detection

### 5. Test Across Devices

Glass effects can vary significantly across:
- Different screen sizes
- Various browsers
- Mobile vs desktop
- Light vs dark themes

## Common Patterns

### Glass Card

```tsx
<div className="bg-white/10 backdrop-blur-lg border border-white/20 rounded-xl p-6 shadow-xl">
  {/* Content */}
</div>
```

### Glass Navigation Bar

```tsx
<nav className="bg-white/5 backdrop-blur-md border-b border-white/10 sticky top-0 z-50">
  {/* Navigation items */}
</nav>
```

### Glass Modal

```tsx
<div className="fixed inset-0 bg-black/50 backdrop-blur-sm">
  <div className="bg-white/10 backdrop-blur-lg border border-white/20 rounded-2xl p-8">
    {/* Modal content */}
  </div>
</div>
```

See component templates in `assets/templates/` for complete implementations.

## Decision Tree

### Choosing Glass Intensity

```
Subtle UI element? → Subtle (backdrop-blur-sm, bg-opacity-5)
Standard card/modal? → Medium (backdrop-blur-lg, bg-opacity-10)
Hero/featured element? → Strong (backdrop-blur-xl, bg-opacity-15)
```

### Web vs Mobile

```
Web application? → Use Tailwind CSS backdrop-blur utilities
React Native app? → Use BlurView from @react-native-community/blur
Need Tailwind in RN? → Use NativeWind + conditional blur
```

### Framework Selection

```
Next.js/React with Tailwind? → Use Tailwind utilities directly
Using Shadcn UI? → Customize components with glass classes
React Native? → Use BlurView component
React Native + Tailwind? → NativeWind + BlurView combination
```

## Reference Files

Detailed documentation available in `references/`:

- **[web-patterns.md](references/web-patterns.md)**: Tailwind CSS patterns, Shadcn UI integration, component examples
- **[mobile-patterns.md](references/mobile-patterns.md)**: React Native implementation, BlurView usage, NativeWind integration
- **[performance.md](references/performance.md)**: Optimization strategies, performance testing, mobile considerations

## Component Templates

Ready-to-use templates in `assets/templates/`:

### Web Templates
- `web/GlassCard.tsx` - Reusable glass card
- `web/GlassButton.tsx` - Button variants
- `web/GlassModal.tsx` - Modal component
- `web/GlassNavbar.tsx` - Navigation bar
- `web/GlassContainer.tsx` - Container with background

### Mobile Templates
- `mobile/GlassCard.tsx` - React Native card
- `mobile/GlassScreen.tsx` - Full screen overlay
- `mobile/GlassModal.tsx` - Modal component
- `mobile/GlassHeader.tsx` - Navigation header

Copy and customize templates for your project needs.

## Troubleshooting

### Blur Not Working

**Web:**
- Check browser support for `backdrop-filter`
- Verify Tailwind config includes backdrop utilities
- Ensure element has transparent/semi-transparent background

**Mobile:**
- Verify BlurView is properly installed
- Check React Native version compatibility
- Ensure BlurView has proper styling (width/height)

### Performance Issues

- Reduce blur intensity
- Limit number of glass elements on screen
- Use `transform` instead of `position` for animations
- Test on target devices

### Readability Problems

- Increase text contrast
- Adjust background opacity
- Use stronger borders
- Consider dark mode variants

See [performance.md](references/performance.md) for detailed troubleshooting.
