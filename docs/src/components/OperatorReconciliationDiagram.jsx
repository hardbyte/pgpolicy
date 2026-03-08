export function OperatorReconciliationDiagram() {
  const steps = [
    {
      step: '1',
      title: 'Read policy and Secret',
      body: 'Load the PostgresPolicy, fetch DATABASE_URL from the referenced Secret, and refresh the cached pool when credentials change.',
      tone: 'sky',
    },
    {
      step: '2',
      title: 'Build desired state',
      body: 'Convert the CRD to the shared PolicyManifest model, then expand profiles and schemas into concrete roles, grants, and memberships.',
      tone: 'indigo',
    },
    {
      step: '3',
      title: 'Inspect PostgreSQL',
      body: 'Query the live database state that matters for this policy, including managed roles, privileges, memberships, and provider-specific constraints.',
      tone: 'amber',
    },
    {
      step: '4',
      title: 'Diff and safety checks',
      body: 'Compute the convergent change plan, detect conflicts, and enforce per-database locking before any mutation is attempted.',
      tone: 'rose',
    },
    {
      step: '5',
      title: 'Apply in one transaction',
      body: 'Execute the rendered SQL statements inside a single transaction so the reconcile either commits fully or rolls back cleanly.',
      tone: 'emerald',
    },
    {
      step: '6',
      title: 'Patch status and emit telemetry',
      body: 'Write conditions, summaries, and last-error state back to Kubernetes, and export OTLP metrics for runtime visibility.',
      tone: 'cyan',
    },
  ]

  return (
    <div className="not-prose my-10">
      <div className="grid gap-4 lg:grid-cols-3">
        {steps.map((step, index) => (
          <ReconcileCard
            key={step.step}
            {...step}
            arrow={index < steps.length - 1}
          />
        ))}
      </div>
    </div>
  )
}

function ReconcileCard({ step, title, body, tone, arrow }) {
  const tones = {
    sky: 'from-sky-100 to-white border-sky-200 dark:from-sky-950/50 dark:to-slate-900 dark:border-sky-900/60',
    indigo:
      'from-indigo-100 to-white border-indigo-200 dark:from-indigo-950/40 dark:to-slate-900 dark:border-indigo-900/60',
    amber:
      'from-amber-100 to-white border-amber-200 dark:from-amber-950/30 dark:to-slate-900 dark:border-amber-900/60',
    rose: 'from-rose-100 to-white border-rose-200 dark:from-rose-950/30 dark:to-slate-900 dark:border-rose-900/60',
    emerald:
      'from-emerald-100 to-white border-emerald-200 dark:from-emerald-950/30 dark:to-slate-900 dark:border-emerald-900/60',
    cyan: 'from-cyan-100 to-white border-cyan-200 dark:from-cyan-950/30 dark:to-slate-900 dark:border-cyan-900/60',
  }

  return (
    <div className="relative">
      <div
        className={`h-full rounded-3xl border bg-gradient-to-br p-5 shadow-lg shadow-slate-900/5 dark:shadow-none ${tones[tone]}`}
      >
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-slate-900 text-sm font-bold text-white dark:bg-white dark:text-slate-900">
            {step}
          </div>
          <p className="m-0 font-display text-xl text-slate-900 dark:text-white">{title}</p>
        </div>
        <p className="mt-4 text-sm leading-6 text-slate-700 dark:text-slate-300">{body}</p>
      </div>
      {arrow ? (
        <div className="pointer-events-none absolute -bottom-3 left-1/2 hidden -translate-x-1/2 lg:block xl:hidden">
          <ArrowDown />
        </div>
      ) : null}
    </div>
  )
}

function ArrowDown() {
  return (
    <svg
      aria-hidden="true"
      viewBox="0 0 24 24"
      className="h-6 w-6 text-slate-400 dark:text-slate-500"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d="M12 5v14" />
      <path d="m6 13 6 6 6-6" />
    </svg>
  )
}
