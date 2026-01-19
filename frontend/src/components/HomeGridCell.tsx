import { Link } from "react-router-dom";

export default function HomeGridCell({
  title,
  description,
  to,
}: {
  title?: string;
  description?: string;
  to: string;
}) {
  return (
    <Link
      to={to}
      className="bg-gradient-to-br from-blue-950/50 to-black/80 backdrop-blur border-2
                       border-2 border-blue-500/60 p-8 rounded-2xl
                       transition-all duration-300 transform hover:border-blue-400 hover:shadow-lg hover:shadow-blue-500/70 hover:scale-105"
    >
      <h2 className="text-2xl font-bold text-white mb-2 group-hover:text-indigo-400 transition-colors">
        {title}
      </h2>
      <p className="text-slate-400 leading-relaxed">{description}</p>
    </Link>
  );
}
