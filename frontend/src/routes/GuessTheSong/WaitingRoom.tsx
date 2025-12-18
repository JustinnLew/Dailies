import type { Player } from "../../utils/types";

export default function Waiting({ lobbyCode, ready, players }
: { lobbyCode: string, ready: () => void, players: Map<string, Player> }
) {

    return (<div className="h-screen flex p-4 bg-gray-100">
		{/* Left panel: Game ID + Player List */}
		<div className="flex flex-col w-1/3 bg-white p-4 rounded shadow-md">
			<div className="mb-4">
			<h2 className="text-lg font-bold">Game Code</h2>
			<p className="text-gray-700">{lobbyCode}</p>
			</div>

			<div>
			<h2 className="text-lg font-bold mb-2">Players</h2>
			<ul className="space-y-2">
				{[...players.entries()].map(([id, p]) => (
					<li key={id} className={`p-2 rounded transition-colors ${p.ready ? "bg-green-200 text-green-900" : "bg-gray-200 text-gray-900"}`}>
					{p.username}
					</li>
				))}
			</ul>
			</div>
		</div>

		{/* Right panel: Settings */}
		<div className="flex-1 ml-4 flex flex-col justify-between bg-white p-4 rounded shadow-md">
			<div>
			<h2 className="text-lg font-bold mb-4">Settings</h2>
			{/* Example settings */}
			<div className="mb-2">
				<label className="block mb-1">Difficulty:</label>
				<select className="w-full border rounded p-1">
				<option>Easy</option>
				<option>Medium</option>
				<option>Hard</option>
				</select>
			</div>
			<div className="mb-2">
				<label className="block mb-1">Number of Songs:</label>
				<input
				type="number"
				className="w-full border rounded p-1"
				defaultValue={5}
				/>
			</div>
			</div>

			{/* Start Game button at bottom-right */}
			<div className="flex justify-end">
			<button className="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600" onClick={ready}>
				Ready
			</button>
			</div>
		</div>
		</div>
    )
}