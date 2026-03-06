function LogomarkPaths() {
  return (
    <g fill="none" strokeLinejoin="round" strokeWidth={1}>
      {/* Database cylinder icon */}
      <ellipse cx="12" cy="7" rx="8" ry="3" style={{fill: "#2563EB", stroke: "#1d4ed8", strokeWidth: 0.5}} />
      <path d="M4 7v10c0 1.66 3.58 3 8 3s8-1.34 8-3V7" style={{fill: "#2563EB", stroke: "#1d4ed8", strokeWidth: 0.5}} />
      <ellipse cx="12" cy="12" rx="8" ry="3" style={{fill: "none", stroke: "rgba(255,255,255,0.4)", strokeWidth: 0.5}} />
      <ellipse cx="12" cy="17" rx="8" ry="3" style={{fill: "none", stroke: "rgba(255,255,255,0.4)", strokeWidth: 0.5}} />
      {/* Shield / lock overlay */}
      <path d="M12 9l4 2v3c0 2.2-1.8 4-4 4s-4-1.8-4-4v-3l4-2z" style={{fill: "rgba(255,255,255,0.9)", stroke: "none"}} />
      <circle cx="12" cy="14" r="1" style={{fill: "#2563EB"}} />
      <path d="M12 14v1.5" style={{stroke: "#2563EB", strokeWidth: 0.8, strokeLinecap: "round"}} />
    </g>
  )
}

export function Logomark(props) {
  return (
    <svg aria-hidden="true" viewBox="0 0 24 24" fill="none" {...props}>
      <LogomarkPaths />
    </svg>
  )
}

export function Logo(props) {
  return (
    <svg aria-hidden="true" viewBox="0 0 140 24" fill="none" {...props}>
      <LogomarkPaths />
      <text x={28} y={17} style={{fontWeight: "normal", fontFamily: "Inter, sans-serif", fontSize: "14px"}}>pgroles</text>
    </svg>
  )
}
