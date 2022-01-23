
export type Input = {
  port1: Gamepad | Unplugged,
  port2: Gamepad | Unplugged,
};

export type Unplugged = {
  type: 'unplugged',
};

export type Gamepad = {
  type: 'gamepad',
  a: boolean,
  b: boolean,
  select: boolean,
  start: boolean,
  up: boolean,
  down: boolean,
  left: boolean,
  right: boolean,
};

export type InputMapping = {
  port1: GamepadMapping | UnpluggedMapping,
  port2: GamepadMapping | UnpluggedMapping,
};

export type UnpluggedMapping = {
  type: 'unplugged',
};

export type GamepadMapping = {
  type: 'gamepad',
  a: string,
  b: string,
  select: string,
  start: string,
  up: string,
  down: string,
  left: string,
  right: string,
};
