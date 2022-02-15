import React, { ChangeEvent, MouseEvent } from 'react';
import { GamepadInput, GamepadInputMapping, InputMapping, InputMappings } from './types';

import './InputScreen.css';
import { InputContext } from './InputProvider';

type InputScreenProps = {
  inputMappings: InputMappings,
  onInputMappingsChange?: (inputMapping: InputMappings) => void,
  onClose?: () => void,
}

const DEFAULT_GAMEPAD_MAPPING = {
  a: 'keyboard.key.g',
  b: 'keyboard.key.f',
  select: 'keyboard.key.r',
  start: 'keyboard.key.t',
  up: 'keyboard.key.w',
  down: 'keyboard.key.s',
  left: 'keyboard.key.a',
  right: 'keyboard.key.d',
};

type GamepadScan = {
  current: 'a' | 'b' | 'select' | 'start' | 'up' | 'down' | 'left' | 'right',
  usedPaths: Set<string>,
  a: null | string,
  b: null | string,
  select: null | string,
  start: null | string,
  up: null | string,
  down: null | string,
  left: null | string,
  right: null | string,
}

export default function InputScreen({
  inputMappings,
  onInputMappingsChange,
  onClose
}: InputScreenProps) {

  let [remapping, setRemapping] = React.useState(false);

  return (
    <div className="inputScreenContainer">
      <PortInput
        name="1"
        remapping={remapping}
        setRemapping={setRemapping}
        inputMapping={inputMappings.port1}
        onInputMappingChange={im => onInputMappingsChange?.({ ...inputMappings, port1: im })}
      />
      <PortInput
        name="2"
        remapping={remapping}
        setRemapping={setRemapping}
        inputMapping={inputMappings.port2}
        onInputMappingChange={im => onInputMappingsChange?.({ ...inputMappings, port2: im })}
      />
      <button type="button" onClick={onClose}>CLOSE</button>
    </div>
  );
}

type PortInputProps = {
  name: string,
  remapping: boolean,
  setRemapping?: (remap: boolean) => void,
  inputMapping: InputMapping,
  onInputMappingChange?: (inputMapping: InputMapping) => void,
}

function PortInput({
  name,
  remapping,
  setRemapping,
  inputMapping,
  onInputMappingChange
}: PortInputProps) {

  let input = React.useContext(InputContext);
  let [gamepadScan, setGamepadScan] = React.useState<GamepadScan | null>(null);
  let [gamepadState, setGamepadState] = React.useState<GamepadInput | null>((() => {
    if (inputMapping.type === 'gamepad') {
      return {
        type: 'gamepad',
        a: input.isPressed(inputMapping.a),
        b: input.isPressed(inputMapping.b),
        select: input.isPressed(inputMapping.select),
        start: input.isPressed(inputMapping.start),
        up: input.isPressed(inputMapping.up),
        down: input.isPressed(inputMapping.down),
        left: input.isPressed(inputMapping.left),
        right: input.isPressed(inputMapping.right),
      };
    }
    return null;
  })());

  function handleMappingTypeChange(e: ChangeEvent<HTMLSelectElement>) {
    setGamepadScan(null);
    setRemapping?.(false);

    let value = e.target.value as unknown as InputMapping["type"];
    if (value === 'unplugged') {
      onInputMappingChange?.({ type: 'unplugged' });
    }
    if (value === 'gamepad') {
      onInputMappingChange?.({ type: 'gamepad', ...DEFAULT_GAMEPAD_MAPPING });
    }
  }

  function handleGamepadRemapClick(e: MouseEvent<HTMLButtonElement>) {
    setGamepadScan({
      current: 'left',
      usedPaths: new Set(),
      a: null,
      b: null,
      select: null,
      start: null,
      up: null,
      down: null,
      left: null,
      right: null,
    });
    setRemapping?.(true);
  }

  function handleGamepadRemapCancelClick(e: MouseEvent<HTMLButtonElement>) {
    setGamepadScan(null);
    setRemapping?.(false);
  }

  React.useEffect(() => {
    if (gamepadScan) {
      let id = { current: 0 };

      function handleAnimationFrame() {
        id.current = window.requestAnimationFrame(handleAnimationFrame);

        let path = input.firstPressedExcept(gamepadScan!.usedPaths);

        if (path !== null) {
          if (gamepadScan!.current === 'a') {
            setGamepadScan(null);
            setRemapping?.(false);
            onInputMappingChange?.({
              type: 'gamepad',
              a: path,
              b: gamepadScan!.b!,
              select: gamepadScan!.select!,
              start: gamepadScan!.start!,
              up: gamepadScan!.up!,
              down: gamepadScan!.down!,
              left: gamepadScan!.left!,
              right: gamepadScan!.right!,
            });
          } else {
            setGamepadScan({
              ...gamepadScan!,
              [gamepadScan!.current]: path,
              usedPaths: (() => {
                const usedPaths = new Set(gamepadScan!.usedPaths);
                usedPaths.add(path);
                return usedPaths;
              })(),
              current: (() => {
                switch (gamepadScan!.current) {
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
      }
      id.current = window.requestAnimationFrame(handleAnimationFrame);
      return () => window.cancelAnimationFrame(id.current);
    }
  }, [gamepadScan, inputMapping, onInputMappingChange, setRemapping, input]);

  React.useEffect(() => {
    if (inputMapping.type === 'gamepad') {
      let id = { current: 0 };

      function handleAnimationFrame() {
        id.current = window.requestAnimationFrame(handleAnimationFrame);

        let gamepadMapping = inputMapping as unknown as GamepadInputMapping;
        let newState: GamepadInput = {
          type: 'gamepad',
          a: input.isPressed(gamepadMapping.a),
          b: input.isPressed(gamepadMapping.b),
          select: input.isPressed(gamepadMapping.select),
          start: input.isPressed(gamepadMapping.start),
          up: input.isPressed(gamepadMapping.up),
          down: input.isPressed(gamepadMapping.down),
          left: input.isPressed(gamepadMapping.left),
          right: input.isPressed(gamepadMapping.right),
        };
        if (gamepadState) {
          for (const key in gamepadState) {
            let k = key as unknown as keyof GamepadInput;
            if (gamepadState[k] !== newState[k]) {
              setGamepadState(newState);
              break;
            }
          }
        } else {
          setGamepadState(newState);
        }
      }
      id.current = window.requestAnimationFrame(handleAnimationFrame);
      return () => window.cancelAnimationFrame(id.current);
    }
  }, [inputMapping, gamepadState, input]);

  function buttonHighlight(button: keyof GamepadInput) {
    if (gamepadScan) {
      return gamepadScan.current === button;
    }
    else if (gamepadState && !remapping) {
      return gamepadState[button];
    }
    else {
      return false;
    }
  }

  return (
    <div>
      <span>Port {name}: </span>
      <select value={inputMapping.type} onChange={handleMappingTypeChange}>
        <option value="unplugged">Unplugged</option>
        <option value="gamepad">Gamepad</option>
      </select>
      {inputMapping.type === 'gamepad' && !remapping && (
        <button type="button" onClick={handleGamepadRemapClick}>Remap</button>
      )}
      {inputMapping.type === 'gamepad' && remapping && gamepadScan && (
        <button type="button" onClick={handleGamepadRemapCancelClick}>Cancel</button>
      )}
      {inputMapping.type === 'gamepad' && remapping && !gamepadScan && (
        <button type="button" disabled>Remap</button>
      )}
      {inputMapping.type === 'gamepad' && (
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
    </div>
  );
}
