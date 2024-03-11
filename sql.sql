drop table player;
create table player (
	player_id serial primary key,
	username text unique,
	password text,
	created_on timestamp default now()
);

drop table session;
create table session (
	session_id serial primary key,
	player1_username text,
	player2_username text,
	result text,
	created_on timestamp default now()
);