import React, { ChangeEvent, MouseEvent } from 'react';
import { Gamepad, InputMapping, GamepadMapping } from './types';

import './InputScreen.css';

type Props = {
  inputMapping: InputMapping,
  onInputMappingChange?: (inputMapping: InputMapping) => void,
  onClose?: () => void,
}

const DEFAULT_GAMEPAD_MAPPING = {
  a: 'keyboard.g',
  b: 'keyboard.f',
  select: 'keyboard.r',
  start: 'keyboard.t',
  up: 'keyboard.w',
  down: 'keyboard.s',
  left: 'keyboard.a',
  right: 'keyboard.d',
};

type GamepadScan = {
  current: 'a' | 'b' | 'select' | 'start' | 'up' | 'down' | 'left' | 'right',
  usedKeys: { [key: string]: undefined },
  a: null | string,
  b: null | string,
  select: null | string,
  start: null | string,
  up: null | string,
  down: null | string,
  left: null | string,
  right: null | string,
}

export default function InputScreen({ inputMapping, onInputMappingChange, onClose }: Props) {
  let [gamepad1Scan, setGamepad1Scan] = React.useState<GamepadScan | null>(null);
  let [gamepad1State, setGamepad1State] = React.useState<Gamepad | null>((() => {
    if (inputMapping.port1.type === 'gamepad') {
      return {
        type: 'gamepad',
        a: window.isPressed(inputMapping.port1.a),
        b: window.isPressed(inputMapping.port1.b),
        select: window.isPressed(inputMapping.port1.select),
        start: window.isPressed(inputMapping.port1.start),
        up: window.isPressed(inputMapping.port1.up),
        down: window.isPressed(inputMapping.port1.down),
        left: window.isPressed(inputMapping.port1.left),
        right: window.isPressed(inputMapping.port1.right),
      };
    }
    return null;
  })());

  function handlePort1TypeChange(e: ChangeEvent<HTMLSelectElement>) {
    let value = e.target.value as unknown as typeof inputMapping.port1.type;
    if (value === 'unplugged') {
      setGamepad1Scan(null);
      onInputMappingChange?.({ port1: { type:'unplugged' }, port2: inputMapping.port2 });
    }
    if (value === 'gamepad') {
      setGamepad1Scan(null);
      onInputMappingChange?.({ port1: { type:'gamepad', ...DEFAULT_GAMEPAD_MAPPING }, port2: inputMapping.port2 });
    }
  }

  function handleGamepad1RemapClick(e: MouseEvent<HTMLButtonElement>) {
    setGamepad1Scan({
      current: 'left',
      usedKeys: {},
      a: null,
      b: null,
      select: null,
      start: null,
      up: null,
      down: null,
      left: null,
      right: null,
    });
  }

  React.useEffect(() => {
    if (gamepad1Scan) {
      let id = { current: 0 };
      function handleAnimationFrame() {
        function setKey(key: string) {
          if (gamepad1Scan!.current === 'a') {
            onInputMappingChange?.({
              port1: {
                type: 'gamepad',
                a: key,
                b: gamepad1Scan!.b!,
                select: gamepad1Scan!.select!,
                start: gamepad1Scan!.start!,
                up: gamepad1Scan!.up!,
                down: gamepad1Scan!.down!,
                left: gamepad1Scan!.left!,
                right: gamepad1Scan!.right!,
              },
              port2: inputMapping.port2,
            });
            setGamepad1Scan(null);
          } else {
            setGamepad1Scan({
              ...gamepad1Scan!,
              [gamepad1Scan!.current]: key,
              usedKeys: {...gamepad1Scan!.usedKeys, [key]: undefined},
              current: (() => {
                switch (gamepad1Scan!.current) {
                  case 'left': return 'up';
                  case 'up': return 'right';
                  case 'right': return 'down';
                  case 'down': return 'select';
                  case 'select': return 'start';
                  case 'start': return 'b';
                  case 'b': return 'a';
                }
              })(),
            });
          }
        }
        buttons_scan:
        do {
          for (const key in window.keyboardState) {
            if (window.keyboardState[key]) {
              const k = `keyboard.${key}`;
              if (!(k in gamepad1Scan!.usedKeys)) {
                setKey(k);
                break buttons_scan;
              }
            }
          }
          for (const gamepad of window.navigator.getGamepads()) {
            if (!gamepad) continue;
            for (const [index, button] of gamepad.buttons.entries()) {
              if (button.pressed || button.value > 0.9) {
                const k: string = `gamepad.${gamepad.id.replaceAll('.', '')}.button.${index}`;
                if (!(k in gamepad1Scan!.usedKeys)) {
                  setKey(k);
                  break buttons_scan;
                }
              }
            }
          }
        } while (false);
        id.current = window.requestAnimationFrame(handleAnimationFrame);
      }
      id.current = window.requestAnimationFrame(handleAnimationFrame);
      return () => window.cancelAnimationFrame(id.current);
    }
  }, [gamepad1Scan, inputMapping, onInputMappingChange]);

  React.useEffect(() => {
    if (inputMapping.port1.type === 'gamepad') {
      let id = { current: 0 };
      function handleAnimationFrame() {
        let gamepadMapping = inputMapping.port1 as unknown as GamepadMapping;
        let newState: Gamepad = {
          type: 'gamepad',
          a: window.isPressed(gamepadMapping.a),
          b: window.isPressed(gamepadMapping.b),
          select: window.isPressed(gamepadMapping.select),
          start: window.isPressed(gamepadMapping.start),
          up: window.isPressed(gamepadMapping.up),
          down: window.isPressed(gamepadMapping.down),
          left: window.isPressed(gamepadMapping.left),
          right: window.isPressed(gamepadMapping.right),
        };
        if (gamepad1State) {
          for (const key in gamepad1State) {
            let k = key as unknown as keyof Gamepad;
            if (gamepad1State[k] !== newState[k]) {
              setGamepad1State(newState);
              break;
            }
          }
        } else {
          setGamepad1State(newState);
        }

        id.current = window.requestAnimationFrame(handleAnimationFrame);
      }
      id.current = window.requestAnimationFrame(handleAnimationFrame);
      return () => window.cancelAnimationFrame(id.current);
    }
  }, [inputMapping.port1, gamepad1State]);

  function buttonHighlight(button: keyof Gamepad) {
    if (gamepad1Scan) {
      return gamepad1Scan.current === button;
    }
    else if (gamepad1State) {
      return gamepad1State[button];
    }
    else {
      return false;
    }
  }

  return (
    <div className="inputScreenContainer">
      <div>
        <span>Port 1: </span>
        <select value={inputMapping.port1.type} onChange={handlePort1TypeChange}>
          <option value="unplugged">Unplugged</option>
          <option value="gamepad">Gamepad</option>
        </select>
        {inputMapping.port1.type === 'gamepad' && (
          <button type="button" onClick={handleGamepad1RemapClick} disabled={!!gamepad1Scan}>Remap</button>
        )}
      </div>
      {inputMapping.port1.type === 'gamepad' && (
        <div className="gamepad">
          <div className={`gamepad-button round a ${buttonHighlight('a') ? 'highlight' : ''}`}></div>
          <div className={`gamepad-button round b ${buttonHighlight('b') ? 'highlight' : ''}`}></div>
          <div className={`gamepad-button short select ${buttonHighlight('select') ? 'highlight' : ''}`}></div>
          <div className={`gamepad-button short start ${buttonHighlight('start') ? 'highlight' : ''}`}></div>
          <div className={`gamepad-button directional up ${buttonHighlight('up') ? 'highlight' : ''}`}></div>
          <div className={`gamepad-button directional down ${buttonHighlight('down') ? 'highlight' : ''}`}></div>
          <div className={`gamepad-button directional left ${buttonHighlight('left') ? 'highlight' : ''}`}></div>
          <div className={`gamepad-button directional right ${buttonHighlight('right') ? 'highlight' : ''}`}></div>
        </div>
      )}
      <button type="button" onClick={onClose}>CLOSE</button>
    </div>
  );
}
