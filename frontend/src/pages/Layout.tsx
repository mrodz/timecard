import { Component, Suspense, use, useEffect } from 'react';
import { Outlet } from 'react-router-dom';
import useAuth from "@/lib/useAuth"

type DashboardLoadedProps = {
	currentUser: Promise<unknown>,
}

function LayoutLoaded(props: DashboardLoadedProps) {
	const user = use(props.currentUser)

	useEffect(() => {
		// not signed in
		if (user === null) {
			const params = new URLSearchParams()
			params.set('client_id', import.meta.env.VITE_COGNITO_CLIENT_ID);
			params.set('response_type', 'code');
			params.set('redirect_uri', 'http://localhost:5173/auth/');
			window.location.href = `https://auth.timecard.pro/login?${params.toString()}&scope=aws.cognito.signin.user.admin+email+openid+phone`
		}
	}, [])

	return (
		user === null ? <b>Please sign in to view this resource</b> : <b><Outlet /></b>
	)
}

export default function Layout() {
	const { getCurrentUser } = useAuth()

	return (
		<Suspense fallback={<b>Loading profile...</b>}>
			<LayoutLoaded currentUser={getCurrentUser()} />
		</Suspense>
	)
}