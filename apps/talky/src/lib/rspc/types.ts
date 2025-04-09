import { type Observable } from './UntypedClient';

export type JoinPath<TPath extends string, TNext extends string> = TPath extends ''
  ? TNext
  : `${TPath}.${TNext}`;

export type ProcedureKind = 'query' | 'mutation' | 'subscription';

export type Procedure = {
  kind: ProcedureKind;
  input: unknown;
  output: unknown;
  error: unknown;
};

export type Procedures = {
  [K in string]: Procedure | Procedures;
};

export type Result<Ok, Err> = { status: 'ok'; data: Ok } | { status: 'err'; error: Err };

export type ProcedureResult<P extends Procedure> = Result<P['output'], P['error']>;

export interface SubscriptionObserver<TValue, TError> {
  onStarted: () => void;
  onData: (value: TValue) => void;
  onError: (err: TError) => void;
  onStopped: () => void;
  onComplete: () => void;
}

export type ExecuteArgs = {
  type: ProcedureKind;
  path: string;
  input: unknown;
};
export type ExecuteFn = (args: ExecuteArgs) => Observable<ExeceuteData>;

export type ExeceuteData = {
  result: // eslint-disable-next-line @typescript-eslint/no-explicit-any
  | { type: 'response'; data: any }
    | {
        type: 'error';
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        data: { code: number; data: any };
      };
};

// { code: number; value: any } | null;
// | { type: "event"; data: any }
// | { type: "response"; data: any }
