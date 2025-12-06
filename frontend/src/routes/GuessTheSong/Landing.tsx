    import Navbar from "../../components/NavBar"

    export default function Landing() {
        return (
            <div className="flex flex-col min-h-screen">
                <Navbar />
                <div className="flex flex-col items-center justify-center bg-gray-100, flex-1">
                    <h1 className="text-4xl font-bold mb-8">Guess The Song</h1>
                    <p className="text-lg text-center max-w-xl px-4">
                        Welcome to Guess The Song! Test your music knowledge by listening to short clips of popular songs and trying to identify them. Challenge yourself and see how many you can get right!
                    </p>
                </div>
            </div>
        )
    }