// PA eDocket Enterprise Design System
// Premium Executive UI - Big Law Aesthetic with Silicon Valley Polish

export const designSystem = {
  // ============= COLOR PALETTE =============
  colors: {
    // Primary Brand Colors (Prestigious Deep Navy & Gold)
    primary: {
      50: '#F0F4FF',
      100: '#E0E9FF',
      200: '#C7D7FE',
      300: '#A4BCFD',
      400: '#8098F9',
      500: '#6172F3',  // Primary Brand
      600: '#4E5CE6',
      700: '#3F4CD1',
      800: '#3540B3',
      900: '#2D3599',
      950: '#1E2363',  // Deep Navy
    },

    // Gold Accents (Premium, Expensive Feel)
    gold: {
      50: '#FFFBEB',
      100: '#FEF3C7',
      200: '#FDE68A',
      300: '#FCD34D',
      400: '#FBBF24',
      500: '#F59E0B',  // Primary Gold
      600: '#D97706',
      700: '#B45309',
      800: '#92400E',
      900: '#78350F',
    },

    // Neutrals (Clean, Professional)
    neutral: {
      0: '#FFFFFF',
      50: '#F9FAFB',
      100: '#F3F4F6',
      200: '#E5E7EB',
      300: '#D1D5DB',
      400: '#9CA3AF',
      500: '#6B7280',
      600: '#4B5563',
      700: '#374151',
      800: '#1F2937',
      900: '#111827',
      950: '#030712',
    },

    // Semantic Colors
    success: {
      50: '#F0FDF4',
      500: '#10B981',
      700: '#047857',
      900: '#064E3B',
    },

    warning: {
      50: '#FFFBEB',
      500: '#F59E0B',
      700: '#B45309',
      900: '#78350F',
    },

    danger: {
      50: '#FEF2F2',
      500: '#EF4444',
      700: '#B91C1C',
      900: '#7F1D1D',
    },

    info: {
      50: '#EFF6FF',
      500: '#3B82F6',
      700: '#1D4ED8',
      900: '#1E3A8A',
    },
  },

  // ============= TYPOGRAPHY =============
  typography: {
    fontFamily: {
      display: '"SF Pro Display", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
      body: '"Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
      mono: '"JetBrains Mono", "SF Mono", Menlo, Monaco, "Courier New", monospace',
      legal: '"Crimson Text", "Baskerville", Georgia, serif',  // For documents
    },

    fontSize: {
      xs: '0.75rem',      // 12px
      sm: '0.875rem',     // 14px
      base: '1rem',       // 16px
      lg: '1.125rem',     // 18px
      xl: '1.25rem',      // 20px
      '2xl': '1.5rem',    // 24px
      '3xl': '1.875rem',  // 30px
      '4xl': '2.25rem',   // 36px
      '5xl': '3rem',      // 48px
      '6xl': '3.75rem',   // 60px
    },

    fontWeight: {
      light: 300,
      normal: 400,
      medium: 500,
      semibold: 600,
      bold: 700,
      black: 900,
    },

    lineHeight: {
      tight: 1.25,
      snug: 1.375,
      normal: 1.5,
      relaxed: 1.625,
      loose: 2,
    },

    letterSpacing: {
      tighter: '-0.05em',
      tight: '-0.025em',
      normal: '0',
      wide: '0.025em',
      wider: '0.05em',
      widest: '0.1em',
    },
  },

  // ============= SPACING =============
  spacing: {
    0: '0',
    1: '0.25rem',   // 4px
    2: '0.5rem',    // 8px
    3: '0.75rem',   // 12px
    4: '1rem',      // 16px
    5: '1.25rem',   // 20px
    6: '1.5rem',    // 24px
    8: '2rem',      // 32px
    10: '2.5rem',   // 40px
    12: '3rem',     // 48px
    16: '4rem',     // 64px
    20: '5rem',     // 80px
    24: '6rem',     // 96px
  },

  // ============= BORDER RADIUS =============
  borderRadius: {
    none: '0',
    sm: '0.25rem',   // 4px
    base: '0.5rem',  // 8px
    md: '0.75rem',   // 12px
    lg: '1rem',      // 16px
    xl: '1.5rem',    // 24px
    '2xl': '2rem',   // 32px
    full: '9999px',
  },

  // ============= SHADOWS =============
  shadows: {
    sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    base: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
    md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
    lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
    xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
    '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
    inner: 'inset 0 2px 4px 0 rgba(0, 0, 0, 0.06)',
    // Premium Glow Effects
    glow: '0 0 20px rgba(97, 114, 243, 0.3)',
    goldGlow: '0 0 20px rgba(245, 158, 11, 0.3)',
  },

  // ============= BREAKPOINTS =============
  breakpoints: {
    xs: '475px',
    sm: '640px',
    md: '768px',
    lg: '1024px',
    xl: '1280px',
    '2xl': '1536px',
  },

  // ============= Z-INDEX =============
  zIndex: {
    hide: -1,
    base: 0,
    dropdown: 1000,
    sticky: 1100,
    fixed: 1200,
    modalBackdrop: 1300,
    modal: 1400,
    popover: 1500,
    tooltip: 1600,
    notification: 1700,
  },

  // ============= TRANSITIONS =============
  transitions: {
    fast: '150ms cubic-bezier(0.4, 0, 0.2, 1)',
    base: '200ms cubic-bezier(0.4, 0, 0.2, 1)',
    slow: '300ms cubic-bezier(0.4, 0, 0.2, 1)',
    slower: '500ms cubic-bezier(0.4, 0, 0.2, 1)',
  },

  // ============= COMPONENT STYLES =============
  components: {
    // Executive Dashboard Card
    card: {
      base: `
        background: white;
        border: 1px solid #E5E7EB;
        border-radius: 1rem;
        box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
        transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
      `,
      hover: `
        box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
        transform: translateY(-2px);
      `,
      premium: `
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        border: none;
        color: white;
      `,
    },

    // Premium Button
    button: {
      primary: `
        background: linear-gradient(135deg, #6172F3 0%, #3540B3 100%);
        color: white;
        padding: 0.75rem 1.5rem;
        border-radius: 0.5rem;
        font-weight: 600;
        font-size: 1rem;
        box-shadow: 0 4px 6px -1px rgba(97, 114, 243, 0.3);
        transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
        border: none;
        cursor: pointer;
      `,
      primaryHover: `
        box-shadow: 0 10px 15px -3px rgba(97, 114, 243, 0.4);
        transform: translateY(-2px);
      `,
      secondary: `
        background: white;
        color: #6172F3;
        padding: 0.75rem 1.5rem;
        border-radius: 0.5rem;
        font-weight: 600;
        font-size: 1rem;
        border: 2px solid #6172F3;
        cursor: pointer;
        transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
      `,
      gold: `
        background: linear-gradient(135deg, #F59E0B 0%, #D97706 100%);
        color: white;
        padding: 0.75rem 1.5rem;
        border-radius: 0.5rem;
        font-weight: 600;
        font-size: 1rem;
        box-shadow: 0 4px 6px -1px rgba(245, 158, 11, 0.3);
        transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
        border: none;
        cursor: pointer;
      `,
    },

    // Input Fields
    input: {
      base: `
        width: 100%;
        padding: 0.75rem 1rem;
        border: 2px solid #E5E7EB;
        border-radius: 0.5rem;
        font-size: 1rem;
        transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
        background: white;
        color: #111827;
      `,
      focus: `
        border-color: #6172F3;
        box-shadow: 0 0 0 3px rgba(97, 114, 243, 0.1);
        outline: none;
      `,
      error: `
        border-color: #EF4444;
        box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
      `,
    },

    // Navigation
    nav: {
      base: `
        background: #1E2363;
        height: 100vh;
        width: 280px;
        padding: 1.5rem;
        color: white;
        box-shadow: 4px 0 10px rgba(0, 0, 0, 0.1);
      `,
      item: `
        padding: 0.75rem 1rem;
        border-radius: 0.5rem;
        cursor: pointer;
        transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
        display: flex;
        align-items: center;
        gap: 0.75rem;
        font-weight: 500;
      `,
      itemHover: `
        background: rgba(97, 114, 243, 0.2);
      `,
      itemActive: `
        background: linear-gradient(135deg, #6172F3 0%, #3540B3 100%);
        box-shadow: 0 4px 6px -1px rgba(97, 114, 243, 0.3);
      `,
    },

    // Table
    table: {
      base: `
        width: 100%;
        border-collapse: separate;
        border-spacing: 0;
      `,
      header: `
        background: #F9FAFB;
        border-bottom: 2px solid #E5E7EB;
        font-weight: 600;
        color: #374151;
        text-align: left;
        padding: 1rem;
        font-size: 0.875rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
      `,
      row: `
        border-bottom: 1px solid #E5E7EB;
        transition: all 200ms cubic-bezier(0.4, 0, 0.2, 1);
      `,
      rowHover: `
        background: #F9FAFB;
      `,
      cell: `
        padding: 1rem;
        color: #111827;
      `,
    },

    // Badge
    badge: {
      success: `
        background: #D1FAE5;
        color: #065F46;
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.875rem;
        font-weight: 600;
      `,
      warning: `
        background: #FEF3C7;
        color: #92400E;
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.875rem;
        font-weight: 600;
      `,
      danger: `
        background: #FEE2E2;
        color: #991B1B;
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.875rem;
        font-weight: 600;
      `,
      info: `
        background: #DBEAFE;
        color: #1E40AF;
        padding: 0.25rem 0.75rem;
        border-radius: 9999px;
        font-size: 0.875rem;
        font-weight: 600;
      `,
    },

    // Modal
    modal: {
      overlay: `
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(4px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1400;
      `,
      content: `
        background: white;
        border-radius: 1rem;
        box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
        max-width: 90vw;
        max-height: 90vh;
        overflow: auto;
        padding: 2rem;
      `,
    },
  },

  // ============= ANIMATIONS =============
  animations: {
    fadeIn: `
      @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
      }
    `,
    slideIn: `
      @keyframes slideIn {
        from {
          opacity: 0;
          transform: translateY(20px);
        }
        to {
          opacity: 1;
          transform: translateY(0);
        }
      }
    `,
    pulse: `
      @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
      }
    `,
    spin: `
      @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
      }
    `,
  },
};

// ============= UTILITY FUNCTIONS =============

export const cn = (...classes: (string | undefined | null | false)[]) => {
  return classes.filter(Boolean).join(' ');
};

export const rgba = (hex: string, alpha: number): string => {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
};

// ============= RESPONSIVE UTILITIES =============

export const responsive = {
  mobile: `@media (max-width: ${designSystem.breakpoints.sm})`,
  tablet: `@media (min-width: ${designSystem.breakpoints.sm}) and (max-width: ${designSystem.breakpoints.lg})`,
  desktop: `@media (min-width: ${designSystem.breakpoints.lg})`,
  wide: `@media (min-width: ${designSystem.breakpoints['2xl']})`,
};

// ============= THEME VARIANTS =============

export const themes = {
  light: {
    background: designSystem.colors.neutral[0],
    surface: designSystem.colors.neutral[50],
    text: designSystem.colors.neutral[900],
    textSecondary: designSystem.colors.neutral[600],
    border: designSystem.colors.neutral[200],
  },
  dark: {
    background: designSystem.colors.neutral[900],
    surface: designSystem.colors.neutral[800],
    text: designSystem.colors.neutral[0],
    textSecondary: designSystem.colors.neutral[400],
    border: designSystem.colors.neutral[700],
  },
};

export default designSystem;
