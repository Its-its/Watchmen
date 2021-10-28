import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

import { DashboardComponent } from './dashboard/dashboard.component';

import { FeedListComponent } from './feeds/feed-list/feed-list.component';
import { WebsitesComponent as FeedWebsitesComponent } from './feeds/websites/websites.component';

const routes: Routes = [
	{ path: 'dashboard', component: DashboardComponent },

	{ path: 'feeds', component: FeedListComponent },
	{ path: 'feeds/watching', component: FeedWebsitesComponent },

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