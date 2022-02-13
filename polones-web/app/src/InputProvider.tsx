import React from 'react';

export type InputTools = {
  isPressed(path: string): boolean,
  firstPressedExcept(except: Set<string>): string | null,
}

export const InputContext = React.createContext<InputTools>({
  isPressed: () => false,
  firstPressedExcept: () => null,
});

export default function InputProvider(props: { children: any }) {
  let keyboardStateRef = React.useRef<Map<string, boolean>>(new Map());

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== "Unidentified" && event.key !== "Dead") {
      keyboardStateRef.current.set(event.key, true);
    }
  }

  function onkeyup(event: KeyboardEvent) {
    if (event.key !== "Unidentified" && event.key !== "Dead") {
      keyboardStateRef.current.set(event.key, false);
    }
  }

  function onblur(_event: FocusEvent) {
    for (const key in keyboardStateRef.current) {
      keyboardStateRef.current.set(key, false);
    }
  }

  function isPressed(path: string): boolean {
    let segments = path.split('.');
    if (segments[0] === 'keyboard') {
      if (segments[1] === 'key') {
        return !!keyboardStateRef.current.get(segments[2]) ?? false;
      }
    }
    if (segments[0] === 'gamepad') {
      for (const gamepad of window.navigator.getGamepads()) {
        if (gamepad && gamepad.id.replaceAll('.', '') === segments[1]) {
          if (segments[2] === 'button') {
            let index = Number(segments[3]);
            if (index < gamepad.buttons.length) {
              return !!gamepad.buttons[index].pressed || gamepad.buttons[index].value > 0.9;
            }
          }
          else if (segments[2] === 'axis') {
            let index = Number(segments[3]);
            if (index < gamepad.axes.length) {
              if (segments[4] === 'positive') {
                return gamepad.axes[index] > 0.5;
              }
              else if (segments[4] === 'negative') {
                return gamepad.axes[index] < -0.5;
              }
            }
          }
        }
      }
    }
    return false;
  }

  function firstPressedExcept(except: Set<string>): string | null {
    for (const key in keyboardStateRef.current) {
      if (keyboardStateRef.current.get(key) ?? false) {
        const k = `keyboard.key.${key}`;
        if (!except.has(k)) {
          return k;
        }
      }
    }
    for (const gamepad of window.navigator.getGamepads()) {
      if (!gamepad) continue;
      for (const [index, button] of gamepad.buttons.entries()) {
        if (button.pressed || button.value > 0.9) {
          const k: string = `gamepad.${gamepad.id.replaceAll('.', '')}.button.${index}`;
          if (!except.has(k)) {
            return k;
          }
        }
      }
      for (const [index, value] of gamepad.axes.entries()) {
        if (value > 0.5) {
          const k: string = `gamepad.${gamepad.id.replaceAll('.', '')}.axis.${index}.positive`;
          if (!except.has(k)) {
            return k;
          }
        }
        else if (value < -0.5) {
          const k: string = `gamepad.${gamepad.id.replaceAll('.', '')}.axis.${index}.negative`;
          if (!except.has(k)) {
            return k;
          }
        }
      }
    }
    return null;
  }

  React.useEffect(() => {
    window.addEventListener('keyup', onkeyup);
    window.addEventListener('keydown', onkeydown);
    window.addEventListener('blur', onblur);
    return () => {
      window.removeEventListener('keyup', onkeyup);
      window.removeEventListener('keydown', onkeydown);
      window.removeEventListener('blur', onblur);
    };
  }, []);

  return (
    <InputContext.Provider value={{ isPressed, firstPressedExcept }}>
      {props.children}
    </InputContext.Provider>
  )
}
