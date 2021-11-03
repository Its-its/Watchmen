import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

import { DashboardComponent } from './dashboard/dashboard.component';

import { FeedListComponent } from './feeds/feed-list/feed-list.component';
import { EditorComponent as FeedEditorComponent } from './feeds/editor/editor.component';
import { FilterEditComponent as FeedFilterEditComponent } from './feeds/filter-edit/filter-edit.component';
import { WebsitesComponent as FeedWebsitesComponent } from './feeds/websites/websites.component';

import { ListComponent as ChangesListComponent } from './changes/list/list.component';


const routes: Routes = [
	{ path: 'dashboard', component: DashboardComponent },

	{ path: 'feeds', component: FeedListComponent },
	{ path: 'feeds/watching', component: FeedWebsitesComponent },
	{ path: 'feeds/filter', component: FeedFilterEditComponent },
	{ path: 'feeds/editor', component: FeedEditorComponent },

	{ path: 'changes', component: ChangesListComponent },

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