import logoUrl from "../assets/spotify-full-logo-white.svg";

export function SpotifyLogo() {
  return (
    <span className="spotify-logo">
      <img src={logoUrl} alt="Spotify" draggable={false} />
    </span>
  );
}
