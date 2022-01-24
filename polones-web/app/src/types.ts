
export type Inputs = {
  port1: Input,
  port2: Input,
};

export type Input = UnpluggedInput | GamepadInput;

export type UnpluggedInput = {
  type: 'unplugged',
};

export type GamepadInput = {
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

export type InputMappings = {
  port1: InputMapping,
  port2: InputMapping,
};

export type InputMapping = UnpluggedInputMapping | GamepadInputMapping;

export type UnpluggedInputMapping = {
  type: 'unplugged',
};

export type GamepadInputMapping = {
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
