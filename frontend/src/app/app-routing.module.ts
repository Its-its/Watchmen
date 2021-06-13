import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

import { DashboardComponent } from './dashboard/dashboard.component';
import { FeedListComponent } from './feed-list/feed-list.component';

const routes: Routes = [
	{ path: 'dashboard', component: DashboardComponent },
	{ path: 'feeds', component: FeedListComponent },

	{ path: '',   redirectTo: '/dashboard', pathMatch: 'full' }
];

@NgModule({
	imports: [
		RouterModule.forRoot(routes)
	],

	exports: [
		RouterModule
	]
})

export class AppRoutingModule { }