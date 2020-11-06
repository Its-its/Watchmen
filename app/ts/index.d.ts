
type Obj<I> = { [name: string]: I };

type Nullable<I> = I | null;
type Optional<I> = I | undefined;


declare namespace rust {
	type Values = string | number | boolean | null;

	export type Optional<T> = T | EnumNone;

	export type EnumValue = rust.EnumNone | EnumObject | Values | ObjectType | EnumValue[];
	export type EnumObject = {
		[name: string]: EnumValue
	}
	export type EnumNone = "None";

	export type ObjectType = {
		[name: string]: EnumValue;
	}

	export type BaseEnum = {
		name: string
		value: EnumValue
	}
}


interface SocketResponse {
	[name: string]: any;

	message_id?: number;
	error?: string;
	result?: {
		method: string;
		params: { [name: string]: any; };
	};
}

type ResponseFunc<V> = (error?: any, value?: V, method?: string) => any;

interface AwaitingReponse {
	sent: number,
	timeout_seconds: number,

	msg_id: number,
	resp_func?: ResponseFunc<any>
}

// Models
interface ModelCategory {
	id?: number;

	date_added: number;
	name: string;
	name_lowercase: string;
	position: number;
}

interface ModelEditCategory {
	date_added?: number;
	name?: string;
	name_lowercase?: string;
	position?: number;
}

interface ModelListener {
	id?: number;

	title: string;
	url: string;
	description: string;
	date_added: number;
	generator: string;
	ignore_if_not_new: boolean;
	global_show: boolean;
	last_called: number;
	remove_after: number;
	sec_interval: number;
}

interface ModelEditListener {
	title?: string;
	description?: string;
	generator?: string;

	ignore_if_not_new?: boolean;
	global_show?: boolean;

	remove_after?: number;
	sec_interval?: number;
}

interface ModelItem {
	id?: number;

	guid: string;
	title: string;
	author: string;
	content: string;
	link: string;
	date: number;
	hash: string;

	date_added: number;
	is_read: boolean;
	is_starred: boolean;
	is_removed: boolean;
	tags: string;
	feed_id: number;
}

interface ModelFeedCategory {
	id?: number;

	feed_id: number;
	category_id: number;
}

interface ModelCustomItem {
	id?: number;

	title: string;
	description: string;
	match_url: string;

	search_opts: {
		[name: string]: Nullable<{
			xpath: string
			parse_type: rust.EnumValue
		} | string>
	}
}

interface ModelEditCustomItem {
	title?: string;
	description?: string;
	match_url?: string;

	search_opts?: {
		[name: string]: Nullable<{
			xpath: string
			parse_type: rust.EnumValue
		}>
	}
}

interface FilterModel {
	id: number;
	title: string;
	filter: rust.EnumObject;
}

interface FilterGroupListener {
	feeds: number[];
	filter: FilterModel;
}

// Responses

interface RemoveListenerResponse {
	//
}

interface EditListenerResponse {
	affected: number;
	listener: ModelEditListener;
}

interface ItemListResponse {
	item_count: number;
	skip_count: number;

	total_items: number;

	items: ModelItem[];
	notification_ids: number[];
}

interface FilterListResponse {
	items: FilterGroupListener[];
}

interface FeedListResponse {
	items: ModelListener[];
}

interface CreateListenerResponse {
	affected: number;
	listener: ModelListener;
}

interface UpdatesResponse {
	new_items: number;
	since: number;
}

interface CreateCategoryResponse {
	affected: number;
	category: ModelCategory;
}

interface CategoryListResponse {
	categories: ModelCategory[];
	category_feeds: ModelFeedCategory[];
}

interface AddCategoryFeedResponse {
	affected: number;
	category: ModelFeedCategory
}

interface GetWebpageResponse {
	html: string;
}

interface CreateCustomItemResponse {
	affected: number;
	category: ModelCustomItem;
}

interface CustomItemListResponse {
	items: ModelCustomItem[];
}



//


interface WatcherListResponse {
	items: ModelWatcher[];
}

interface ModelWatcher {
	id?: number;

	title: string;
	url: string;
	description: string;

	remove_after: number;
	sec_interval: number;

	last_called: number;
	date_added: number;
}