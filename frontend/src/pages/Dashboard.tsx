import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { use, Suspense, memo } from "react";
import { CurrentUserContext } from "./Layout";
import ClockList from "@/components/ClockList";
import { Spinner } from "@/components/ui/spinner";
import { loadAllUserClocks } from "@/lib/api/clocks";

const ClockLoading = memo(() => {
	return (
		<div className="flex items-center justify-center">
			<Spinner />
		</div>
	)
})

export default function Dashboard() {
	const user = use(CurrentUserContext)!

	return (
		<div>
			<Card className="m-8 p-8">
				<CardTitle className="mb-4">
					Clocks
				</CardTitle>
				<CardContent>
					<Suspense fallback={<ClockLoading />}>
						<ClockList loadAllClocks={loadAllUserClocks({ userPoolId: user.reactiveUser!.getUsername()! })} />
					</Suspense>
				</CardContent>
			</Card>
		</div>
	)
}