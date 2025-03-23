import type { Procedures, LobbyData as LobbyDataI } from '@feid/bindings';

export type LobbyData = Procedures['lobby_create']['output'];

export type LobbyPresenceData = LobbyDataI;
