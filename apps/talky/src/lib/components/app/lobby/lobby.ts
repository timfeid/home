import type { Procedures } from '@feid/bindings';

export type LobbyData = Procedures['lobby_create']['output'];

export type LobbyPresenceData = Procedures['lobby_subscribe']['output'];
