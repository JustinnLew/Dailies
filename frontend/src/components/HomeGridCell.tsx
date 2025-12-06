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
            className="bg-white p-6 rounded-lg shadow-md hover:shadow-lg transition-shadow block"
        >
            <h2 className="text-xl font-semibold mb-4">{title}</h2>
            <p className="text-gray-600">{description}</p>
        </Link>
    );
}
