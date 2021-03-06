let {
  admin,
  adminKeys,
  adminSettings,

  project,
  projectKeys,
  projectSettings,
} = require('../data/projects')

let { email } = require('../data/settings')
let { adminUser, getUsers } = require('../data/users')

exports.seed = async function(knex) {
  console.log('Delete Projects')
  await knex('projects').del()
 
  console.log('Insert Projects')
  await knex('projects').insert([
    admin,
    project
  ])

  console.log('Insert Settings')
  await knex('project_settings').insert([
    adminSettings,
    projectSettings,
  ])

  console.log('Insert Email Settings')
  await knex('email_settings').insert([email])

  console.log('Insert Keys')
  await knex('project_keys').insert([
    adminKeys,
    projectKeys
  ])


  console.log('Insert users')
  await knex('users').insert([
    ...getUsers(1000),
    adminUser,
  ])
};
