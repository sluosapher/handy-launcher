export type CompletionAction = {
  label: string;
  disabled: boolean;
  action: 'open' | 'download';
};

export type CompletionActionSnapshot = {
  handyInstalled: boolean;
  handyRunning: boolean;
};

export function resolveCompletionAction(
  snapshot: CompletionActionSnapshot
): CompletionAction {
  if (snapshot.handyRunning) {
    return {
      label: 'Handy is already open',
      disabled: true,
      action: 'open'
    };
  }

  if (snapshot.handyInstalled) {
    return {
      label: 'Open Handy',
      disabled: false,
      action: 'open'
    };
  }

  return {
    label: 'Download Handy',
    disabled: false,
    action: 'download'
  };
}
