-- Add up migration script here
alter table users
    add column username varchar(20) not null default 'username';

update users set username = name where username is not null;

alter table users
    add column password text not null default '$2a$12$l6eh6td3qp0nmipSnq/pbeUAuwuZCrbkJJaiobSyXW9AakoYtMY92';