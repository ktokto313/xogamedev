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
	board varchar(1)[3][3],
	created_on timestamp default now()
);

insert into player(username, password) values ('kto', 'kto');
insert into player(username, password) values ('kto1', 'kto1');