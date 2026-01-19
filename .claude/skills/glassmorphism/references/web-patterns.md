# Web Glassmorphism Patterns

Comprehensive patterns for implementing glassmorphic effects in web applications using Tailwind CSS and Shadcn UI.

## Table of Contents

- [Tailwind CSS Utilities](#tailwind-css-utilities)
- [Component Patterns](#component-patterns)
- [Shadcn UI Integration](#shadcn-ui-integration)
- [Responsive Patterns](#responsive-patterns)
- [Dark Mode Support](#dark-mode-support)
- [Advanced Patterns](#advanced-patterns)

## Tailwind CSS Utilities

### Core Utilities

**Backdrop Blur:**
- `backdrop-blur-none` - No blur
- `backdrop-blur-sm` - Small blur (4px)
- `backdrop-blur` - Default blur (8px)
- `backdrop-blur-md` - Medium blur (12px)
- `backdrop-blur-lg` - Large blur (16px)
- `backdrop-blur-xl` - Extra large blur (24px)
- `backdrop-blur-2xl` - 2X large blur (40px)
- `backdrop-blur-3xl` - 3X large blur (64px)

**Background Opacity:**
- `bg-white/5` - 5% opacity
- `bg-white/10` - 10% opacity (recommended)
- `bg-white/15` - 15% opacity
- `bg-white/20` - 20% opacity
- `bg-black/10` - Dark glass variant

**Border Opacity:**
- `border-white/10` - Subtle border
- `border-white/20` - Standard border
- `border-white/30` - Prominent border

**Shadows:**
- `shadow-lg` - Large shadow
- `shadow-xl` - Extra large shadow
- `shadow-2xl` - 2X large shadow

### Complete Glass Class Combination

```tsx
// Standard glass effect
className="bg-white/10 backdrop-blur-lg border border-white/20 rounded-xl shadow-xl"

// Dark glass variant
className="bg-black/10 backdrop-blur-lg border border-white/10 rounded-xl shadow-xl"

// Strong glass effect
className="bg-white/15 backdrop-blur-xl border border-white/30 rounded-2xl shadow-2xl"
```

## Component Patterns

### Glass Card

**Basic:**
```tsx
<div className="bg-white/10 backdrop-blur-lg border border-white/20 rounded-xl p-6 shadow-xl">
  <h3 className="text-white font-semibold mb-2">Card Title</h3>
  <p className="text-white/80">Card content goes here</p>
</div>
```

**With Hover Effect:**
```tsx
<div className="
  bg-white/10 backdrop-blur-lg 
  border border-white/20 
  rounded-xl p-6 shadow-xl
  transition-all duration-300
  hover:bg-white/15 hover:border-white/30 hover:shadow-2xl
">
  {/* Content */}
</div>
```

**Interactive Card:**
```tsx
<div className="
  bg-white/10 backdrop-blur-lg 
  border border-white/20 
  rounded-xl p-6 shadow-xl
  cursor-pointer
  transition-all duration-300
  hover:bg-white/15 
  hover:scale-105
  active:scale-95
">
  {/* Content */}
</div>
```

### Glass Button

**Primary:**
```tsx
<button className="
  bg-white/10 backdrop-blur-md
  border border-white/20
  rounded-lg px-6 py-3
  text-white font-medium
  shadow-lg
  transition-all duration-200
  hover:bg-white/15 hover:border-white/30
  active:scale-95
">
  Click Me
</button>
```

**Secondary:**
```tsx
<button className="
  bg-transparent
  border-2 border-white/30
  rounded-lg px-6 py-3
  text-white font-medium
  backdrop-blur-sm
  transition-all duration-200
  hover:bg-white/10 hover:backdrop-blur-md
  active:scale-95
">
  Secondary
</button>
```

### Glass Modal

```tsx
{/* Backdrop */}
<div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center">
  {/* Modal */}
  <div className="
    bg-white/10 backdrop-blur-xl
    border border-white/20
    rounded-2xl p-8
    shadow-2xl
    max-w-md w-full mx-4
  ">
    <h2 className="text-white text-2xl font-bold mb-4">Modal Title</h2>
    <p className="text-white/80 mb-6">Modal content</p>
    <div className="flex gap-4 justify-end">
      <button className="px-4 py-2 text-white/80 hover:text-white">Cancel</button>
      <button className="px-4 py-2 bg-white/20 rounded-lg hover:bg-white/30">Confirm</button>
    </div>
  </div>
</div>
```

### Glass Navigation Bar

**Fixed Top:**
```tsx
<nav className="
  fixed top-0 left-0 right-0
  bg-white/5 backdrop-blur-md
  border-b border-white/10
  z-50
  px-6 py-4
">
  <div className="max-w-7xl mx-auto flex items-center justify-between">
    <div className="text-white font-bold text-xl">Logo</div>
    <div className="flex gap-6">
      <a href="#" className="text-white/80 hover:text-white">Home</a>
      <a href="#" className="text-white/80 hover:text-white">About</a>
      <a href="#" className="text-white/80 hover:text-white">Contact</a>
    </div>
  </div>
</nav>
```

**Sticky:**
```tsx
<nav className="
  sticky top-0
  bg-white/5 backdrop-blur-md
  border-b border-white/10
  z-50
  px-6 py-4
">
  {/* Navigation content */}
</nav>
```

### Glass Sidebar

```tsx
<aside className="
  fixed left-0 top-0 bottom-0
  w-64
  bg-white/10 backdrop-blur-lg
  border-r border-white/20
  p-6
  overflow-y-auto
">
  {/* Sidebar content */}
</aside>
```

### Glass Input

```tsx
<input
  type="text"
  className="
    w-full
    bg-white/10 backdrop-blur-md
    border border-white/20
    rounded-lg px-4 py-3
    text-white placeholder:text-white/50
    focus:outline-none
    focus:border-white/40
    focus:bg-white/15
    transition-all duration-200
  "
  placeholder="Enter text..."
/>
```

## Shadcn UI Integration

### Customizing Shadcn Components

**Glass Card:**
```tsx
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

<Card className="bg-white/10 backdrop-blur-lg border-white/20 shadow-xl">
  <CardHeader>
    <CardTitle className="text-white">Glass Card</CardTitle>
  </CardHeader>
  <CardContent className="text-white/80">
    Content with glass effect
  </CardContent>
</Card>
```

**Glass Button:**
```tsx
import { Button } from "@/components/ui/button";

<Button 
  variant="outline"
  className="
    bg-white/10 backdrop-blur-md
    border-white/20
    text-white
    hover:bg-white/15 hover:border-white/30
  "
>
  Glass Button
</Button>
```

**Glass Dialog:**
```tsx
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";

<Dialog>
  <DialogContent className="
    bg-white/10 backdrop-blur-xl
    border-white/20
    text-white
  ">
    <DialogHeader>
      <DialogTitle className="text-white">Glass Dialog</DialogTitle>
    </DialogHeader>
    {/* Content */}
  </DialogContent>
</Dialog>
```

**Glass Sheet:**
```tsx
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";

<Sheet>
  <SheetContent className="
    bg-white/10 backdrop-blur-xl
    border-white/20
    text-white
  ">
    <SheetHeader>
      <SheetTitle className="text-white">Glass Sheet</SheetTitle>
    </SheetHeader>
    {/* Content */}
  </SheetContent>
</Sheet>
```

### Creating Glass Variants

Add custom variants to your Shadcn components:

```tsx
// components/ui/card.tsx
const cardVariants = cva(
  "rounded-lg border bg-card text-card-foreground shadow-sm",
  {
    variants: {
      variant: {
        default: "bg-card",
        glass: "bg-white/10 backdrop-blur-lg border-white/20",
        "glass-strong": "bg-white/15 backdrop-blur-xl border-white/30",
      }
    },
    defaultVariants: {
      variant: "default",
    },
  }
);
```

## Responsive Patterns

### Breakpoint Adjustments

```tsx
<div className="
  bg-white/10 backdrop-blur-md
  md:backdrop-blur-lg
  lg:backdrop-blur-xl
  p-4 md:p-6 lg:p-8
  rounded-lg md:rounded-xl lg:rounded-2xl
">
  {/* Content */}
</div>
```

### Mobile-First Glass

```tsx
<div className="
  bg-white/5 backdrop-blur-sm
  sm:bg-white/10 sm:backdrop-blur-md
  md:bg-white/10 md:backdrop-blur-lg
  border border-white/10
  sm:border-white/20
  p-4 sm:p-6
  rounded-lg sm:rounded-xl
">
  {/* Content */}
</div>
```

## Dark Mode Support

### Using Tailwind Dark Mode

```tsx
<div className="
  bg-white/10 dark:bg-black/10
  backdrop-blur-lg
  border border-white/20 dark:border-white/10
  text-gray-900 dark:text-white
">
  {/* Content */}
</div>
```

### Conditional Glass Styles

```tsx
<div className="
  bg-white/10 dark:bg-white/5
  backdrop-blur-lg dark:backdrop-blur-md
  border border-white/20 dark:border-white/10
">
  {/* Content */}
</div>
```

## Advanced Patterns

### Animated Glass

```tsx
<div className="
  bg-white/10 backdrop-blur-lg
  border border-white/20
  rounded-xl p-6
  shadow-xl
  transition-all duration-500
  hover:backdrop-blur-xl
  hover:bg-white/15
  hover:shadow-2xl
  animate-pulse
">
  {/* Content */}
</div>
```

### Gradient Glass Background

```tsx
<div className="relative">
  {/* Gradient background */}
  <div className="absolute inset-0 bg-gradient-to-br from-purple-500 to-pink-500" />
  
  {/* Glass overlay */}
  <div className="relative bg-white/10 backdrop-blur-lg border border-white/20 rounded-xl p-6">
    {/* Content */}
  </div>
</div>
```

### Layered Glass Effect

```tsx
<div className="relative">
  {/* Base layer */}
  <div className="bg-white/5 backdrop-blur-sm rounded-xl p-6">
    {/* Middle layer */}
    <div className="bg-white/10 backdrop-blur-md rounded-lg p-4 -mt-2">
      {/* Top layer */}
      <div className="bg-white/15 backdrop-blur-lg rounded-md p-2 -mt-1">
        {/* Content */}
      </div>
    </div>
  </div>
</div>
```

### Glass with Image Background

```tsx
<div className="relative overflow-hidden rounded-xl">
  {/* Background image */}
  <img 
    src="/background.jpg" 
    alt="Background"
    className="absolute inset-0 w-full h-full object-cover"
  />
  
  {/* Glass overlay */}
  <div className="relative bg-white/10 backdrop-blur-lg border border-white/20 p-6">
    {/* Content */}
  </div>
</div>
```

## Feature Detection

### CSS Feature Query

```css
@supports (backdrop-filter: blur(10px)) {
  .glass {
    backdrop-filter: blur(10px);
  }
}

@supports not (backdrop-filter: blur(10px)) {
  .glass {
    background-color: rgba(255, 255, 255, 0.3);
  }
}
```

### JavaScript Detection

```tsx
const supportsBackdropFilter = CSS.supports('backdrop-filter', 'blur(10px)');

<div className={supportsBackdropFilter ? 'backdrop-blur-lg' : 'bg-white/30'}>
  {/* Content */}
</div>
```

## Performance Tips

1. **Use `will-change` for animated glass:**
```tsx
<div className="backdrop-blur-lg will-change-[backdrop-filter]">
```

2. **Limit blur intensity on mobile:**
```tsx
<div className="backdrop-blur-md md:backdrop-blur-lg lg:backdrop-blur-xl">
```

3. **Use `transform` instead of `position` for animations:**
```tsx
<div className="backdrop-blur-lg transition-transform hover:scale-105">
```

4. **Reduce number of glass elements on screen:**
   - Use glass effects selectively
   - Consider lazy loading for off-screen glass elements
