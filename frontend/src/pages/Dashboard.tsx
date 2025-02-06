import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { useCallback, use, Suspense, memo } from "react";
import { CurrentUserContext } from "./Layout";
import { loadAllUserClocks } from "@/lib/api";
import ClockList from "@/components/ClockList";
import { Spinner } from "@/components/ui/spinner";

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
			Dashboard!!!!!

			<Card className="m-8 p-8">
				<CardTitle>
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